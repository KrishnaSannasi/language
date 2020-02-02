
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Reg(pub usize);

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

use std::num::NonZeroU64;
use std::rc::Rc;
use std::cell::Cell;

// impl Inference {
//     #[allow(clippy::new_without_default)]
//     pub fn new() -> Self {
//         use std::sync::atomic::{AtomicU64, Ordering::Relaxed};
//         static INDEX: AtomicU64 = AtomicU64::new(0);

//         Self(Rc::new(InferenceInner {
//             resolved: Cell::new(false),
//             id: NonZeroU64::new(1 + INDEX.fetch_add(1, Relaxed))
//                 .expect("tried to create too many inference variables"),
//         }))
//     }

//     pub fn id(&self) -> NonZeroU64 {
//         self.0.id
//     }

//     pub fn resolve(&self) {
//         self.0.resolved.set(true);
//     }

//     pub fn is_resolved(&self) -> bool {
//         self.0.resolved.get()
//     }
// }

// struct InferenceInner {
//     id: NonZeroU64,
//     resolved: Cell<bool>
// }

// #[derive(Clone, PartialEq, Eq, Hash)]
// pub struct Inference(Rc<InferenceInner>);

// use std::fmt;

// impl fmt::Debug for Inference {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         f.debug_struct("Inference")
//             .field("id", &self.0.id)
//             .finish()
//     }
// }

// impl Eq for InferenceInner {}
// impl PartialEq for InferenceInner {
//     fn eq(&self, other: &InferenceInner) -> bool {
//         std::ptr::eq(self, other)
//     }
// }

// impl std::hash::Hash for InferenceInner {
//     fn hash<H: std::hash::Hasher>(&self, hasher: &mut H) {
//         std::ptr::hash(self, hasher)
//     }
// }

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    Primitive(Primitive),
    Inf(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Void {}

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