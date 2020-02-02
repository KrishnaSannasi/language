use core_types::{Type, Primitive};
use core_mir::{Mir, Reg, Load};

use crate::encode::MirDigest;

pub fn infer_types(mir: &MirDigest) -> Option<Vec<Type>> {
    let mut types = (0..mir.max_reg_count).map(Type::Inf)
        .collect::<Vec<_>>();

    macro_rules! write_type {
        ($reg:ident <- $ty:expr) => {{
            let rty = &mut types[$reg];
            if rty.is_inference() || *rty == $ty || $ty.is_inference() && !rty.is_inference() {
                *rty = $ty;
            } else {
                eprintln!(
                    "TypeError ({}), found type: {:?}, expected {:?}",
                    $reg,
                    rty,
                    $ty,
                );

                return None
            }
        }}
    };
    
    for block in mir.blocks.iter().flatten() {
        for mir in block.mir.iter() {
            match *mir {
                | Mir::Jump(_)
                | Mir::Print(_) => {
                    // no types can be gleaned from a print/jump
                }
                Mir::BranchTrue { cond: Reg(cond), .. } => {
                    // cond must be a bool

                    write_type!(cond <- Type::Primitive(Primitive::Bool));
                }
                Mir::Load { to: Reg(to), from } => match from {
                    Load::Bool(_) => write_type!(to <- Type::Primitive(Primitive::Bool)),
                    | Load::U8(_) | Load::U16(_) => write_type!(to <- Type::Primitive(Primitive::I32)),
                    _ => {
                        eprintln!(
                            "TypeError ({}), found type: {{large integer}}, expected {:?}",
                            to,
                            Type::Primitive(Primitive::I32),
                        );
                        
                        return None
                    }
                }
                Mir::LoadReg { to: Reg(to), from: Reg(from) } => {
                    write_type!(to <- Type::Inf(from))
                }
                Mir::BinOp { .. } => {}
                Mir::PreOp { .. } => {}
            }
        }
    }

    Some(types)
}