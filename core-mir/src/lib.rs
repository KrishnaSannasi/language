
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Reg(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BinOpType {
    Add, Sub, Mul, Div,
    Equal, NotEqual, LessThan, LessThanOrEqual, GreaterThan, GreaterThanOrEqual
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PreOpType {
    Not, Neg,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Mir {
    Jump(usize),
    BranchTrue { cond: Reg, target: usize },
    Load { to: Reg, from: Load },
    LoadReg { to: Reg, from: Reg },
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