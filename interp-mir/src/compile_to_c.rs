use core_mir::{BinOpType, Load, Mir, PreOpType, Reg};
use core_types::{Ty, Type, Variant, Primitive};
use impl_pass_mir::encode::MirDigest;
use std::io::{self, Write};

use crate::variables;

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
    let (assign, layout) = variables::Variables::layout(&types);

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
