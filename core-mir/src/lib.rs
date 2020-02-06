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
                    | &Mir::BranchTrue { target, .. }
                    | &Mir::Jump(target) => {
                        if target >= blocks.len() {
                            eprintln!("target is out of bounds!");
                            return None
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
    Func {
        stack_frame: StackFrame<BMeta, FMeta>
    }
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
