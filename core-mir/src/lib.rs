
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MirNode {

}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Reg(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BinOpType {
    Add, Sub, Mul, Div
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PreOpType {
    Not, Neg,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Mir {
    __,
    NoOp(&'static str),
    Jump(usize),
    BranchTrue { cond: Reg, target: usize },
    Load { to: Reg, from: Load },
    Print(Reg),
    BinOp {
        op: BinOpType,
        out: Reg,
        left: Load,
        right: Load,
    },
    PreOp {
        op: PreOpType,
        out: Reg,
        arg: Load,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Load {
    Reg(Reg),
    Bool(bool),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
}