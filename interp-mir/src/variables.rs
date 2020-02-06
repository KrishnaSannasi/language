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
    pub fn layout(types: &'a [Ty<'idt, 'tcx>]) -> (Vec<usize>, Layout) {
        let mut assign = vec![0; types.len()];
        let mut types = types.to_vec();
        
        // sort by alignemnt, then by size in decreasing order of both
        // This is a simple hueristic that will give the optimal
        // packing when `align <= size` and `size % align == 0`
        // in other cases there may be holes up to size `max_align - 1`
        types.sort_unstable_by(|a, b| {
            a.align().cmp(&b.align())
                .then(a.size.cmp(&b.size))
                .reverse()
        });

        let mut map = HashMap::new();

        for (i, &ty) in types.iter().enumerate() {
            // the order or variable assignments doesn't matter in general
            // but it is easier to test things using a stable output, 
            // so BTreeSet is prefered for testing and `HashSet` is prefered
            // for performace, this difference may matter for a large number of variables
            map.entry(ty)
                .or_insert_with(BTreeSet::new)
                // .or_insert_with(HashSet::new)
                .insert(i);
        }

        let mut size = 0;
        let mut align = 1;
        
        for ty in types {
            if let Some(items) = map.remove(ty) {
                align = align.max(ty.align());
                let mask = ty.align() - 1;

                for pos in items {
                    // fix alignment
                    size = (size + mask) & !mask;
                    assign[pos] = size;
                    size += ty.size;
                }
            }
        }

        (assign, Layout::from_size_align(size, align).unwrap())
    }

    pub fn new(types: &'a [Ty<'idt, 'tcx>]) -> Self {
        let (assign, layout) = Self::layout(types);

        Self {
            types,
            assign,
            // frame: StackFrame::new(layout),
        }
    }

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
