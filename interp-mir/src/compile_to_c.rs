use core_mir::{BinOpType, Load, Mir, PreOpType, Reg};
use core_types::{Ty, Type, Variant, Primitive};
use impl_pass_mir::encode::MirDigest;
use std::io::{self, Write};

use std::alloc::Layout;

pub fn layout(types: &[Ty<'_, '_>]) -> (Vec<usize>, Layout) {
    use std::cmp::Ordering;

    #[derive(Clone, Copy, Eq)]
    struct OrdTy<'idt, 'tcx>(Ty<'idt, 'tcx>);

    impl PartialEq for OrdTy<'_, '_> {
        fn eq(&self, other: &Self) -> bool {
            self.cmp(other) == Ordering::Equal
        }
    }

    impl PartialOrd for OrdTy<'_, '_> {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }

    impl Ord for OrdTy<'_, '_> {
        fn cmp(&self, other: &Self) -> Ordering {
            self.0.align().cmp(&other.0.align())
                .then(self.0.size.cmp(&other.0.size))
                .reverse()
        }
    }

    let mut assign = vec![0; types.len()];
    let mut map = std::collections::BTreeMap::new();

    for (i, &ty) in types.iter().enumerate() {
        // the order or variable assignments doesn't matter in general
        // but it is easier to test things using a stable output, 
        // so `BTreeSet` is prefered for testing and `HashSet` is prefered
        // for performace, this difference may matter for a large number of variables
        map.entry(OrdTy(ty))
            .or_insert_with(std::collections::BTreeSet::new)
            // .or_insert_with(std::collections::HashSet::new)
            .insert(i);
    }

    let mut size = 0;
    let mut align = 1;
    
    for (OrdTy(ty), items) in map {
        // remove so that types don't get emitted twice
        // because types in `types` are not guaranteed to be
        // unique
        align = align.max(ty.align());
        let mask = ty.align() - 1;

        for pos in items {
            // fix alignment
            size = (size + mask) & !mask;
            assign[pos] = size;
            size += ty.size;
        }
    }

    (assign, Layout::from_size_align(size, align).unwrap())
}

pub fn emit_c(digest: MirDigest, mut writer: impl Write, ident: &lib_intern::Interner) -> io::Result<()> {
    write_c(digest, &mut writer, ident)
}

struct GetLocal<'a, 'b> {
    assign: &'a [usize],
    reg: Reg,
    ty: &'b dyn std::fmt::Display,
}

impl std::fmt::Display for GetLocal<'_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "*(({}*)(locals + {}))", self.ty, self.assign[self.reg.0])
    }
}

fn write_c<'idt>(digest: MirDigest, writer: &mut dyn Write, ident: &'idt lib_intern::Interner) -> io::Result<()> {
    macro_rules! emit {
        ($($t:tt)*) => {
            write!(writer, $($t)*)?;
        }
    }

    emit!(
        "\
    #include <stdio.h>\n\
    #include <stdint.h>\n\
    int main() {{\n"
    );

    let ty_ctx = lib_arena::cache::Cache::new();
    let types = impl_pass_mir::type_check::infer_types(
        &digest,
        impl_pass_mir::type_check::Context {
            ident,
            ty: &ty_ctx,
        }
    ).expect("Could not deduce types");
    let (assign, layout) = layout(&types);

    macro_rules! get {
        ($reg:expr, $ty:expr) => {
            GetLocal {
                assign: assign.as_slice(),
                reg: $reg,
                ty: &$ty as _,
            }
        };
    }

    emit!(
        "char locals[{size}] __attribute__((aligned({align})));\n",
        size = layout.size(),
        align = layout.align()
    );

    for (block_idx, block) in digest
        .blocks
        .iter()
        .enumerate()
        .flat_map(|(i, x)| Some((i, x.as_ref()?)))
    {
        emit!("\n_label_{}:\n", block_idx);
        for mir in block.mir.iter() {
            match *mir {
                Mir::Jump(j) => {
                    emit!("goto _label_{};\n", j);
                }
                Mir::BranchTrue { cond, target } => {
                    emit!(
                        "if( {} != 0 ) goto _label_{};\n",
                        get!(cond, "_Bool"),
                        target
                    );
                }
                Mir::Load { from, to } => {
                    let ty = match types[to.0].ty {
                        Variant::Primitive(Primitive::Bool) => "_Bool",
                        Variant::Primitive(Primitive::I32) => "int32_t",
                        _ => unreachable!(),
                    };

                    let value = match from {
                        Load::Bool(x) => i32::from(x),
                        Load::U8(x) => i32::from(x),
                        Load::U16(x) => i32::from(x),
                        _ => unreachable!(),
                    };

                    emit!("{} = {};\n", get!(to, ty), value);
                }
                Mir::LoadReg { from, to } => {
                    let ty = &types[to.0];
                    let ty = match ty.ty {
                        Variant::Primitive(Primitive::Bool) => "_Bool",
                        Variant::Primitive(Primitive::I32) => "int32_t",
                        _ if ty.size == 0 => continue,
                        _ => unreachable!(),
                    };

                    emit!("{} = {};\n", get!(to, ty), get!(from, ty));
                }
                Mir::Print(reg) => {
                    let (fmt_spec, ty) = match types[reg.0].ty {
                        Variant::Primitive(Primitive::Bool) => ("b", "_Bool"),
                        Variant::Primitive(Primitive::I32) => ("d", "int32_t"),
                        _ => unreachable!(),
                    };

                    emit!("printf(\"%{}\\n\", {});\n", fmt_spec, get!(reg, ty));
                }
                Mir::BinOp {
                    op,
                    out,
                    left,
                    right,
                } => match op {
                    BinOpType::Add => emit!(
                        "{} = {} + {};\n",
                        get!(out, "int32_t"),
                        get!(left, "int32_t"),
                        get!(right, "int32_t")
                    ),
                    BinOpType::Sub => emit!(
                        "{} = {} - {};\n",
                        get!(out, "int32_t"),
                        get!(left, "int32_t"),
                        get!(right, "int32_t")
                    ),
                    BinOpType::Mul => emit!(
                        "{} = {} * {};\n",
                        get!(out, "int32_t"),
                        get!(left, "int32_t"),
                        get!(right, "int32_t")
                    ),
                    BinOpType::Div => emit!(
                        "{} = {} / {};\n",
                        get!(out, "int32_t"),
                        get!(left, "int32_t"),
                        get!(right, "int32_t")
                    ),

                    BinOpType::GreaterThan => emit!(
                        "{} = {} > {};\n",
                        get!(out, "int32_t"),
                        get!(left, "int32_t"),
                        get!(right, "int32_t")
                    ),
                    BinOpType::LessThan => emit!(
                        "{} = {} < {};\n",
                        get!(out, "int32_t"),
                        get!(left, "int32_t"),
                        get!(right, "int32_t")
                    ),
                    BinOpType::GreaterThanOrEqual => emit!(
                        "{} = {} >= {};\n",
                        get!(out, "int32_t"),
                        get!(left, "int32_t"),
                        get!(right, "int32_t")
                    ),
                    BinOpType::LessThanOrEqual => emit!(
                        "{} = {} <= {};\n",
                        get!(out, "int32_t"),
                        get!(left, "int32_t"),
                        get!(right, "int32_t")
                    ),

                    BinOpType::Equal => {
                        let ty = match types[left.0].ty {
                            Variant::Primitive(Primitive::Bool) => "_Bool",
                            Variant::Primitive(Primitive::I32) => "int32_t",
                            _ => unreachable!(),
                        };

                        emit!(
                            "{} = {} == {};\n",
                            get!(out, "_Bool"),
                            get!(left, ty),
                            get!(right, ty),
                        )
                    }

                    BinOpType::NotEqual => {
                        let ty = match types[left.0].ty {
                            Variant::Primitive(Primitive::Bool) => "_Bool",
                            Variant::Primitive(Primitive::I32) => "int32_t",
                            _ => unreachable!(),
                        };

                        emit!(
                            "{} = {} != {};\n",
                            get!(out, "_Bool"),
                            get!(left, ty),
                            get!(right, ty),
                        )
                    }
                },
                Mir::PreOp { op, out, arg } => todo!(),
            }
        }

        if block.children.is_empty() {
            emit!("return 0;\n");
        }
    }

    emit!("}}");

    Ok(())
}
