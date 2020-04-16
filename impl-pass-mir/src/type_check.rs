use core_mir::{BinOpType, Load, Mir, Reg};
use core_types::{Primitive, Ty, Type, Variant};

use core_tokens::Ident;
use lib_arena::cache::Cache;
use lib_intern::Interner;

use super::*;

use vec_utils::VecExt;

use std::collections::BTreeMap;
use std::ops::{Index, IndexMut};
use std::sync::atomic::{AtomicU64, Ordering::Relaxed};

static FUNC_ID: AtomicU64 = AtomicU64::new(0);

pub struct Context<'idt, 'tcx> {
    pub ident: &'idt Interner,
    pub ty: &'tcx Cache<Type<'idt, 'tcx>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct InfIdx(usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ProjectIdx(usize);

impl From<Reg> for InfIdx {
    fn from(Reg(reg): Reg) -> Self {
        InfIdx(reg)
    }
}

struct InferenceVariables<'idt, 'tcx>(Vec<Infer<'idt, 'tcx>>);
struct ProjectedVariables<'idt>(Vec<Project<'idt>>);

impl InferenceVariables<'_, '_> {
    pub fn inf(&mut self) -> InfIdx {
        let idx = InfIdx(self.0.len());
        self.0.push(Infer::Inf(idx));
        idx
    }
}

impl<'idt> ProjectedVariables<'idt> {
    pub fn push(&mut self, project: Project<'idt>) -> ProjectIdx {
        let idx = ProjectIdx(self.0.len());
        self.0.push(project);
        idx
    }
}

impl<'idt, 'tcx> Index<InfIdx> for InferenceVariables<'idt, 'tcx> {
    type Output = Infer<'idt, 'tcx>;

    fn index(&self, InfIdx(idx): InfIdx) -> &Self::Output {
        &self.0[idx]
    }
}

impl<'idt, 'tcx> IndexMut<InfIdx> for InferenceVariables<'idt, 'tcx> {
    fn index_mut(&mut self, InfIdx(idx): InfIdx) -> &mut Self::Output {
        &mut self.0[idx]
    }
}

impl<'idt, 'tcx> Index<Reg> for InferenceVariables<'idt, 'tcx> {
    type Output = Infer<'idt, 'tcx>;

    fn index(&self, Reg(idx): Reg) -> &Self::Output {
        &self.0[idx]
    }
}

impl<'idt, 'tcx> IndexMut<Reg> for InferenceVariables<'idt, 'tcx> {
    fn index_mut(&mut self, Reg(idx): Reg) -> &mut Self::Output {
        &mut self.0[idx]
    }
}

impl<'idt> Index<ProjectIdx> for ProjectedVariables<'idt> {
    type Output = Project<'idt>;

    fn index(&self, ProjectIdx(idx): ProjectIdx) -> &Self::Output {
        &self.0[idx]
    }
}

impl<'idt> IndexMut<ProjectIdx> for ProjectedVariables<'idt> {
    fn index_mut(&mut self, ProjectIdx(idx): ProjectIdx) -> &mut Self::Output {
        &mut self.0[idx]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Infer<'idt, 'tcx> {
    Concrete(Ty<'idt, 'tcx>),
    Inf(InfIdx),
    Project(ProjectIdx),
}

struct Project<'idt> {
    inf: InfIdx,
    ty: ProjectTy<'idt>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ProjectTy<'idt> {
    ReturnType(InfIdx),
    Function {
        name: Ident<'idt>,
        captures: BTreeMap<Ident<'idt>, InfIdx>,
        return_type: InfIdx,
    },
}

impl Infer<'_, '_> {
    fn is_inference(&self) -> bool {
        match *self {
            Self::Inf(_) | Self::Project(..) => true,
            _ => false,
        }
    }
}

pub fn infer_types<'tcx, 'idt>(
    frame: &StackFrame,
    ctx: Context<'idt, 'tcx>,
) -> Option<Vec<Ty<'idt, 'tcx>>> {
    let types = (0..frame.meta.max_reg_count)
        .map(InfIdx)
        .map(Infer::Inf)
        .collect::<Vec<_>>();

    let mut types = InferenceVariables(types);
    let mut projections = ProjectedVariables(Vec::new());

    macro_rules! debug {
        ($($rest:tt)*) => {
            if true {
                println!("DEBUG: {}", format_args!($($rest)*))
            }
        };
    }

    macro_rules! register {
        ($(
            $ty_var:ident {
                name: $name:expr,
                size: $size:expr,
                align: $align:expr,
                variant: $variant:expr,
            }
        )*) => {$(
            let $ty_var = ctx.ty.insert(
                Type::new(
                    core_tokens::Ident::new(ctx.ident.insert(&$name)),
                    $variant,
                )
                    .with_size($size)
                    .align_to($align)
            );
        )*};
    }

    register! {
        bool_ty {
            name: "bool",
            size: 1,
            align: 1,
            variant: Variant::Primitive(Primitive::Bool),
        }

        i32_ty {
            name: "i32",
            size: 4,
            align: 4,
            variant: Variant::Primitive(Primitive::I32),
        }

        unit {
            name: "()",
            size: 0,
            align: 1,
            variant: Variant::Struct { fields: None },
        }
    }

    macro_rules! write_type {
        ($reg:ident <- $ty:expr) => {{
            let rty = &mut types[$reg];
            if rty.is_inference() || *rty == $ty || $ty.is_inference() && !rty.is_inference() {
                *rty = $ty;
            } else {
                eprintln!(
                    "TypeError ({}), found type: {:?}, expected {:?}",
                    $reg.0, rty, $ty,
                );

                return None;
            }
        }};
        ($a:ident == $b:ident) => {{
            let (a, b) = ($a, $b);
            if a != b {
                assert!(a.0 < types.0.len());
                assert!(b.0 < types.0.len());

                let (a, b) = unsafe {
                    let types = types.0.as_mut_ptr();

                    (&mut *types.add(a.0), &mut *types.add(b.0))
                };

                match (a.is_inference(), b.is_inference()) {
                    (true, _) => *a = b.clone(),
                    (_, true) => *b = a.clone(),
                    (false, false) => {
                        if a != b {
                            eprintln!("TypeError ({}), found type: {:?}, expected {:?}", $a, b, a,);

                            return None;
                        }
                    }
                }
            }
        }};
    };

    for block in frame.blocks().iter() {
        for mir in block.instructions.iter() {
            match *mir {
                Mir::CallFunction | Mir::Jump(_) | Mir::Print(_) => {
                    // no types can be gleaned from a print/jump
                }
                Mir::BranchTrue { cond, .. } => {
                    // cond must be a bool

                    debug!("{} <- bool (branch condition)", cond);
                    write_type!(cond <- Infer::Concrete(bool_ty));
                }
                Mir::Load { to, from } => match from {
                    Load::Bool(_) => write_type!(to <- Infer::Concrete(bool_ty)),
                    Load::U8(_) | Load::U16(_) => {
                        debug!("{} <- i32 (load immediate)", to);
                        write_type!(to <- Infer::Concrete(i32_ty))
                    }
                    _ => {
                        eprintln!(
                            "TypeError ({}), found type: {{large integer}}, expected {:?}",
                            to,
                            Infer::Concrete(i32_ty),
                        );

                        return None;
                    }
                },
                Mir::LoadReg { to, from } => {
                    debug!("{} == {} (load register)", to, from);
                    write_type!(to == from);
                }
                Mir::BinOp {
                    op,
                    out,
                    left,
                    right,
                } => match op {
                    BinOpType::Add | BinOpType::Sub | BinOpType::Mul | BinOpType::Div => {
                        write_type!(out <- Infer::Concrete(i32_ty));
                        write_type!(left <- Infer::Concrete(i32_ty));
                        write_type!(right <- Infer::Concrete(i32_ty));

                        debug!("{} <- i32 (arith)", out);
                        debug!("{} <- i32 (arith)", left);
                        debug!("{} <- i32 (arith)", right);
                    }
                    BinOpType::LessThan
                    | BinOpType::GreaterThan
                    | BinOpType::LessThanOrEqual
                    | BinOpType::GreaterThanOrEqual => {
                        write_type!(out <- Infer::Concrete(bool_ty));
                        write_type!(left <- Infer::Concrete(i32_ty));
                        write_type!(right <- Infer::Concrete(i32_ty));

                        debug!("{} <- bool (comp)", out);
                        debug!("{} <- i32 (comp)", left);
                        debug!("{} <- i32 (comp)", right);
                    }
                    BinOpType::Equal | BinOpType::NotEqual => {
                        write_type!(out <- Infer::Concrete(bool_ty));
                        write_type!(left == right);

                        debug!("{} <- bool (comp)", out);
                        debug!("{} == {} (comp)", left, right);
                    }
                },
                Mir::PreOp { .. } => {}
                Mir::CreateFunc {
                    binding,
                    ret,
                    ref stack_frame,
                } => {
                    let id = FUNC_ID.fetch_add(1, Relaxed);

                    if id > (u64::max_value() >> 1) {
                        panic!("tried to create too many functions!")
                    }

                    let ret = types.inf();

                    let func_proj = projections.push(Project {
                        inf: InfIdx::from(binding),
                        ty: ProjectTy::Function {
                            name: Ident::new(ctx.ident.insert(&format!("$fn({})", id))),
                            captures: BTreeMap::new(),
                            return_type: ret,
                        },
                    });

                    debug!("{} <- fn (function declaration)", func_proj.0);
                    debug!("{} <- () (unit return type)", ret.0);

                    write_type!(binding <- Infer::Project(func_proj));
                    write_type!(ret <- Infer::Concrete(unit));
                }
                Mir::LoadFunction { func, ret } => {
                    let proj = projections.push(Project {
                        inf: InfIdx::from(ret),
                        ty: ProjectTy::ReturnType(InfIdx::from(func)),
                    });

                    write_type!(ret <- Infer::Project(proj));
                    debug!("{} <- ret({}) (load fn)", ret.0, func.0);
                }
                Mir::PopArgument { arg } => {}
                Mir::PushArguement { arg } => {}
            }
        }
    }

    loop {
        let mut has_changed = false;

        for ty in 0..types.0.len() {
            let ty = InfIdx(ty);
            match types[ty] {
                Infer::Concrete(_) => (),
                Infer::Inf(other) => {
                    if other == ty {
                        eprintln!("Failed to infer type of {}", ty.0);
                    }

                    has_changed |= types[ty] != types[other];
                    types[ty] = types[other];
                }
                Infer::Project(idx) => {
                    let proj = &projections[idx].ty;

                    match *proj {
                        ProjectTy::Function {
                            name: _,
                            ref captures,
                            return_type,
                        } => {
                            assert!(captures.is_empty());

                            todo!("register function");

                            // let $ty_var = ctx.ty.insert(
                            //     Type::new(
                            //         core_tokens::Ident::new(ctx.ident.insert(&$name)),
                            //         $variant,
                            //     )
                            //         .with_size($size)
                            //         .align_to($align)
                            // );

                            // types[ty] = Type::/
                        }
                        ProjectTy::ReturnType(func) => {
                            types[ty] = Infer::Concrete(unit);

                            // if let Infer::Concrete(ty) = types[func] {}
                        }
                    }
                }
            }
        }

        if !has_changed {
            break;
        }
    }

    Some(types.0.map(|x| match x {
        Infer::Concrete(t) => t,
        Infer::Inf(i) => panic!("inference failed at: {}", i.0),
        Infer::Project(i) => panic!("project {} not resolved", i.0),
    }))
}
