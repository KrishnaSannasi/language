use core_mir::{BinOpType, Load, Mir, Reg};
use core_types::{Primitive, Type};

use crate::encode::MirDigest;

pub fn infer_types(mir: &MirDigest) -> Option<Vec<Type>> {
    let mut types = (0..mir.max_reg_count).map(Type::Inf).collect::<Vec<_>>();

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

                    write_type!(cond <- Type::Primitive(Primitive::Bool));
                }
                Mir::Load { to: Reg(to), from } => match from {
                    Load::Bool(_) => write_type!(to <- Type::Primitive(Primitive::Bool)),
                    Load::U8(_) | Load::U16(_) => {
                        write_type!(to <- Type::Primitive(Primitive::I32))
                    }
                    _ => {
                        eprintln!(
                            "TypeError ({}), found type: {{large integer}}, expected {:?}",
                            to,
                            Type::Primitive(Primitive::I32),
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
                        write_type!(out <- Type::Primitive(Primitive::I32));
                        write_type!(left <- Type::Primitive(Primitive::I32));
                        write_type!(right <- Type::Primitive(Primitive::I32));
                    }
                    BinOpType::LessThan
                    | BinOpType::GreaterThan
                    | BinOpType::LessThanOrEqual
                    | BinOpType::GreaterThanOrEqual => {
                        write_type!(out <- Type::Primitive(Primitive::Bool));
                        write_type!(left <- Type::Primitive(Primitive::I32));
                        write_type!(right <- Type::Primitive(Primitive::I32));
                    }
                    BinOpType::Equal | BinOpType::NotEqual => {
                        write_type!(out <- Type::Primitive(Primitive::Bool));
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
            if let Type::Inf(other) = types[ty] {
                if other == ty {
                    eprintln!("Failed to infer type of {}", ty);
                }

                has_changed |= types[ty] != types[other];
                types[ty] = types[other].clone();
            }
        }

        if !has_changed {
            break;
        }
    }

    Some(types)
}
