use std::alloc::Layout;

// use crate::stack_frame::StackFrame;
use core_types::{Primitive, Type, Ty};

use std::collections::{HashMap, HashSet, BTreeSet};

pub struct Variables<'a, 'idt, 'tcx> {
    types: &'a [Ty<'idt, 'tcx>],
    assign: Vec<usize>,
    // frame: StackFrame,
}

impl<'a, 'idt, 'tcx> Variables<'a, 'idt, 'tcx> {
    

    // pub fn new(types: &'a [Ty<'idt, 'tcx>]) -> Self {
    //     let (assign, layout) = Self::layout(types);

    //     Self {
    //         types,
    //         assign,
    //         // frame: StackFrame::new(layout),
    //     }
    // }

    // pub fn get<T>(&self, reg: usize) -> T
    // where
    //     Self: Get<T>,
    // {
    //     Get::get(self, reg)
    // }

    // pub fn set<T>(&mut self, reg: usize, value: T)
    // where
    //     Self: Get<T>,
    // {
    //     *self.get_mut(reg) = value;
    // }

    // pub fn copy(&mut self, from: usize, to: usize) {
    //     let fty = &self.types[from];
    //     let tty = &self.types[to];
    //     assert_eq!(fty, tty);

    //     let from = self.assign[from];
    //     let to = self.assign[to];

    //     let ptr = self.frame.ptr();

    //     let size = match fty {
    //         Type::Primitive(Primitive::Bool) => std::mem::size_of::<bool>(),
    //         Type::Primitive(Primitive::I32) => std::mem::size_of::<i32>(),
    //         _ => unreachable!(),
    //     };

    //     unsafe {
    //         ptr.add(from).copy_to(ptr.add(to), size);
    //     }
    // }
}
