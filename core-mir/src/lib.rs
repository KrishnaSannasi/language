#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Reg(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BinOpType {
    Add,
    Sub,
    Mul,
    Div,
    Equal,
    NotEqual,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PreOpType {
    Not,
    Neg,
}

pub type InstructionList<BMeta, FMeta> = Vec<Mir<BMeta, FMeta>>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Block<BMeta, FMeta> {
    pub instructions: InstructionList<BMeta, FMeta>,
    pub meta: BMeta,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StackFrame<BMeta, FMeta> {
    blocks: Vec<Block<BMeta, FMeta>>,
    pub meta: FMeta,
}

impl<BMeta, FMeta> StackFrame<BMeta, FMeta> {
    pub fn new(blocks: Vec<Block<BMeta, FMeta>>, meta: FMeta) -> Option<Self> {
        for block in &blocks {
            for mir in &block.instructions {
                match mir {
                    &Mir::BranchTrue { target, .. } | &Mir::Jump(target) => {
                        if target >= blocks.len() {
                            eprintln!("target is out of bounds!");
                            return None;
                        }
                    }
                    _ => (),
                }
            }
        }

        Some(Self { blocks, meta })
    }

    pub fn blocks(&self) -> &[Block<BMeta, FMeta>] {
        &self.blocks
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Mir<BMeta, FMeta> {
    Jump(usize),
    BranchTrue {
        cond: Reg,
        target: usize,
    },
    Load {
        to: Reg,
        from: Load,
    },
    LoadReg {
        to: Reg,
        from: Reg,
    },
    Print(Reg),
    BinOp {
        op: BinOpType,
        out: Reg,
        left: Reg,
        right: Reg,
    },
    PreOp {
        op: PreOpType,
        out: Reg,
        arg: Reg,
    },
    CreateFunc {
        binding: Reg,
        ret: Reg,
        stack_frame: StackFrame<BMeta, FMeta>,
    },
    LoadFunction {
        func: Reg,
        ret: Reg,
    },
    PopArgument {
        arg: Reg,
    },
    PushArguement {
        arg: Reg,
    },
    CallFunction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Load {
    Bool(bool),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
}

use std::{cell::Cell, fmt};

thread_local! {
    static STACK_DEPTH: Cell<u32> = Cell::new(0)
}

struct Tabs;
impl fmt::Display for Tabs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let depth = STACK_DEPTH.with(|depth| depth.get());

        for _ in 1..depth {
            write!(f, "\t")?;
        }

        Ok(())
    }
}

impl fmt::Display for Reg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "reg({})", self.0)
    }
}

impl<BMeta, FMeta> fmt::Display for StackFrame<BMeta, FMeta> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let blocks = self.blocks().iter().enumerate();

        STACK_DEPTH.with(|depth| {
            depth.set(depth.get() + 1);
        });

        for (i, block) in blocks.clone() {
            writeln!(f, "{}BLOCK({})", Tabs, i)?;
            for (i, mir) in block.instructions.iter().enumerate() {
                writeln!(f, "{}{:3}: {}", Tabs, i, mir)?;
            }
            writeln!(f, "{}ENDBLOCK({})", Tabs, i)?;
        }

        STACK_DEPTH.with(|depth| {
            depth.set(depth.get() - 1);
        });

        Ok(())
    }
}

impl<BMeta, FMeta> fmt::Display for Mir<BMeta, FMeta> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Jump(target) => write!(f, "jmp {}", target),
            Self::BranchTrue { cond, target } => write!(f, "branch {} to {}", cond, target),
            Self::Load { to, from } => match from {
                Load::Bool(from) => write!(f, "load(bool) {} {}", to, from),
                Load::U8(from) => write!(f, "load(u8) {} {}", to, from),
                Load::U16(from) => write!(f, "load(u16) {} {}", to, from),
                Load::U32(from) => write!(f, "load(u32) {} {}", to, from),
                Load::U64(from) => write!(f, "load(u64) {} {}", to, from),
                Load::U128(from) => write!(f, "load(u128) {} {}", to, from),
            },
            Self::LoadReg { to, from } => write!(f, "load(reg) {} {}", to, from),
            Self::Print(Reg(value)) => write!(f, "print {}", value),
            Self::BinOp {
                op,
                out,
                left,
                right,
            } => write!(f, "bin({:?}) {}, {}, {}", op, out, left, right),
            Self::PreOp { op, out, arg } => write!(f, "bin({:?}) {}, {}", op, out, arg),
            Self::CreateFunc {
                binding,
                ret,
                ref stack_frame,
            } => {
                writeln!(f, "fn {} -> {}", binding, ret)?;
                write!(f, "{}", stack_frame)?;
                write!(f, "{}     endfn {} -> {}", Tabs, binding, ret)
            }
            Self::LoadFunction { func, ret } => write!(f, "load(fn) {} -> {}", func, ret),
            Self::PopArgument { arg } => write!(f, "pop(arg) {}", arg),
            Self::PushArguement { arg } => write!(f, "push(arg) {}", arg),
            Self::CallFunction => write!(f, "call"),
        }
    }
}
