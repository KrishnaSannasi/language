
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Reg<'tcx>(pub u64, pub Option<&'tcx Type>);

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
pub enum Mir<'tcx> {
    Jump(usize),
    BranchTrue { cond: Reg<'tcx>, target: usize },
    Load { to: Reg<'tcx>, from: Load },
    LoadReg { to: Reg<'tcx>, from: Reg<'tcx> },
    Print(Reg<'tcx>),
    BinOp {
        op: BinOpType,
        out: Reg<'tcx>,
        left: Reg<'tcx>,
        right: Reg<'tcx>,
    },
    PreOp {
        op: PreOpType,
        out: Reg<'tcx>,
        arg: Reg<'tcx>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Type {
    Primitive(Primitive),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Primitive {
    Bool,
    I32,
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