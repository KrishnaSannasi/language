pub mod encode;
pub mod type_check;

use std::collections::HashSet;

pub type Mir = core_mir::Mir<BlockMeta, FrameMeta>;
pub type Block = core_mir::Block<BlockMeta, FrameMeta>;
pub type StackFrame = core_mir::StackFrame<BlockMeta, FrameMeta>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockMeta {
    pub parents: HashSet<usize>,
    pub children: HashSet<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrameMeta {
    pub max_reg_count: usize,
}
