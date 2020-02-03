use core_mir::{BinOpType, Load, Mir, PreOpType, Reg};
use core_types::{Type, Primitive};
use impl_pass_mir::encode::MirDigest;

mod stack_frame;
mod variables;

pub fn interpret(digest: MirDigest) {
    let types = impl_pass_mir::type_check::infer_types(&digest).expect("Could not deduce types");
    let mut vars = variables::Variables::new(&types);
    
    // let mut mem = vec![0_i64; digest.max_reg_count];

    let blocks = &digest.blocks;
    let mut current_block = blocks[0].as_ref().unwrap().mir.iter();

    loop {
        let mir = match current_block.next() {
            Some(mir) => mir,
            None => return,
        };

        match *mir {
            Mir::Print(Reg(reg)) => match types[reg] {
                Type::Primitive(Primitive::Bool) => println!("{}", vars.get::<bool>(reg)),
                Type::Primitive(Primitive::I32) => println!("{}", vars.get::<i32>(reg)),
                Type::Inf(_) => unreachable!(),
            },
            Mir::Jump(target) => current_block = blocks[target].as_ref().unwrap().mir.iter(),
            Mir::BranchTrue { ref cond, target } => {
                if vars.get::<bool>(cond.0) {
                    current_block = blocks[target].as_ref().unwrap().mir.iter()
                }
            }
            Mir::Load { to: Reg(to), from } => match from {
                Load::Bool(x) => vars.set(to, x),
                Load::U8(x) => vars.set(to, x as i32),
                Load::U16(x) => vars.set(to, x as i32),
                Load::U32(_) => panic!("cannot load 32-bit literals!"),
                Load::U64(_) => panic!("cannot load 64-bit literals!"),
                Load::U128(_) => panic!("cannot load 128-bit literals!"),
            },
            Mir::LoadReg {
                to: Reg(to),
                from: Reg(from),
            } => vars.copy(from, to),
            Mir::BinOp {
                op,
                out: Reg(to),
                left: Reg(left),
                right: Reg(right),
            } => {
                match op {
                    BinOpType::Add => {
                        let out = vars.get::<i32>(left) + vars.get::<i32>(right);
                        vars.set(to, out);
                    },
                    BinOpType::Sub => {
                        let out = vars.get::<i32>(left) - vars.get::<i32>(right);
                        vars.set(to, out);
                    },
                    BinOpType::Mul => {
                        let out = vars.get::<i32>(left) * vars.get::<i32>(right);
                        vars.set(to, out);
                    },
                    BinOpType::Div => {
                        let out = vars.get::<i32>(left) / vars.get::<i32>(right);
                        vars.set(to, out);
                    },

                    BinOpType::Equal => {
                        let out = match types[left] {
                            Type::Primitive(Primitive::Bool) => vars.get::<bool>(left) == vars.get(right),
                            Type::Primitive(Primitive::I32) => vars.get::<i32>(left) == vars.get(right),
                            Type::Inf(_) => unreachable!(),
                        };

                        vars.set(to, out);
                    },
                    BinOpType::NotEqual => {
                        let out = match types[left] {
                            Type::Primitive(Primitive::Bool) => vars.get::<bool>(left) != vars.get(right),
                            Type::Primitive(Primitive::I32) => vars.get::<i32>(left) != vars.get(right),
                            Type::Inf(_) => unreachable!(),
                        };

                        vars.set(to, out);
                    },
                    BinOpType::LessThan => {
                        let out = vars.get::<i32>(left) < vars.get(right);
                        vars.set(to, out);
                    },
                    BinOpType::GreaterThan => {
                        let out = vars.get::<i32>(left) > vars.get(right);
                        vars.set(to, out);
                    },
                    BinOpType::LessThanOrEqual => {
                        let out = vars.get::<i32>(left) <= vars.get(right);
                        vars.set(to, out);
                    },
                    BinOpType::GreaterThanOrEqual => {
                        let out = vars.get::<i32>(left) >= vars.get(right);
                        vars.set(to, out);
                    },
                };
            }
            Mir::PreOp {
                op,
                out: Reg(to),
                arg: Reg(arg),
            } => {
                todo!();
                // let arg = mem[arg];

                // mem[to] = match op {
                //     PreOpType::Not => !arg,
                //     PreOpType::Neg => -arg,
                // };
            }
        }
    }
}
