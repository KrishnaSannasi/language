
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
pub enum Mir {
    Load { to: Reg, from: Load },
    Print(Reg),
    BinOp {
        op: BinOpType,
        out: Reg,
        left: Load,
        right: Load,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Load {
    Reg(Reg),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
}