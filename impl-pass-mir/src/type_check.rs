use core_mir::{BinOpType, Load, Mir, Reg};
use core_types::{Primitive, Ty, Type, Variant};

use lib_arena::cache::Cache;
use lib_intern::Interner;

use crate::encode::MirDigest;

use vec_utils::VecExt;

pub struct Context<'idt, 'tcx> {
    pub ident: &'idt Interner,
    pub ty: &'tcx Cache<Type<'idt>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Infer<'idt, 'tcx> {
    Ty(Ty<'idt, 'tcx>),
    Inf(usize),
}

impl Infer<'_, '_> {
    fn is_inference(&self) -> bool {
        match *self {
            Infer::Inf(_) => true,
            _ => false,
        }
    }
}

pub fn infer_types<'tcx, 'idt>(
    mir: &MirDigest,
    ctx: Context<'idt, 'tcx>,
) -> Option<Vec<Ty<'idt, 'tcx>>> {
    let mut types = (0..mir.max_reg_count).map(Infer::Inf).collect::<Vec<_>>();

    macro_rules! register {
        ($(
            $ty_var:ident = $ty_name:ident {
                size: $size:expr,
                align: $align:expr,
                variant: $variant:expr,
            }
        )*) => {$(
            let $ty_var = ctx.ty.insert(
                Type::new(
                    core_tokens::Ident::new(ctx.ident.insert(stringify!($ty_name))),
                    $variant,
                )
                    .with_size($size)
                    .align_to($align)
            );
        )*};
    }

    register! {
        bool_ty = bool {
            size: 1,
            align: 1,
            variant: Variant::Primitive(Primitive::Bool),
        }

        i32_ty = i32 {
            size: 4,
            align: 4,
            variant: Variant::Primitive(Primitive::I32),
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
                    $reg, rty, $ty,
                );

                return None;
            }
        }};
        ($a:ident == $b:ident) => {{
            let (a, b) = ($a, $b);
            if a != b {
                assert!(a < types.len());
                assert!(b < types.len());

                let (a, b) = unsafe {
                    let types = types.as_mut_ptr();

                    (&mut *types.add(a), &mut *types.add(b))
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

    for block in mir.blocks.iter().flatten() {
        for mir in block.mir.iter() {
            match *mir {
                Mir::Jump(_) | Mir::Print(_) => {
                    // no types can be gleaned from a print/jump
                }
                Mir::BranchTrue {
                    cond: Reg(cond), ..
                } => {
                    // cond must be a bool

                    write_type!(cond <- Infer::Ty(bool_ty));
                }
                Mir::Load { to: Reg(to), from } => match from {
                    Load::Bool(_) => write_type!(to <- Infer::Ty(bool_ty)),
                    Load::U8(_) | Load::U16(_) => write_type!(to <- Infer::Ty(i32_ty)),
                    _ => {
                        eprintln!(
                            "TypeError ({}), found type: {{large integer}}, expected {:?}",
                            to,
                            Infer::Ty(i32_ty),
                        );

                        return None;
                    }
                },
                Mir::LoadReg {
                    to: Reg(to),
                    from: Reg(from),
                } => {
                    write_type!(to == from);
                }
                Mir::BinOp {
                    op,
                    out: Reg(out),
                    left: Reg(left),
                    right: Reg(right),
                } => match op {
                    BinOpType::Add | BinOpType::Sub | BinOpType::Mul | BinOpType::Div => {
                        write_type!(out <- Infer::Ty(i32_ty));
                        write_type!(left <- Infer::Ty(i32_ty));
                        write_type!(right <- Infer::Ty(i32_ty));
                    }
                    BinOpType::LessThan
                    | BinOpType::GreaterThan
                    | BinOpType::LessThanOrEqual
                    | BinOpType::GreaterThanOrEqual => {
                        write_type!(out <- Infer::Ty(bool_ty));
                        write_type!(left <- Infer::Ty(i32_ty));
                        write_type!(right <- Infer::Ty(i32_ty));
                    }
                    BinOpType::Equal | BinOpType::NotEqual => {
                        write_type!(out <- Infer::Ty(bool_ty));
                        write_type!(left == right);
                    }
                },
                Mir::PreOp { .. } => {}
            }
        }
    }

    loop {
        let mut has_changed = false;

        for ty in 0..types.len() {
            if let Infer::Inf(other) = types[ty] {
                if other == ty {
                    eprintln!("Failed to infer type of {}", ty);
                }

                has_changed |= types[ty] != types[other];
                types[ty] = types[other];
            }
        }

        if !has_changed {
            break;
        }
    }

    Some(types.map(|x| match x {
        Infer::Ty(t) => t,
        Infer::Inf(i) => panic!("inference failed at: {}", i),
    }))
}
