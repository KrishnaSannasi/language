use std::alloc::Layout;

use core_types::{Type, Primitive};
use crate::stack_frame::StackFrame;

pub struct Variables<'a> {
    types: &'a [Type],
    assign: Vec<usize>,
    frame: StackFrame,
}

impl<'a> Variables<'a> {
    pub fn new(types: &'a [Type]) -> Self {
        let mut assign = vec![0; types.len()];

        let count_bool = types.iter()
            .inspect(|x| assert!(!x.is_inference()))
            .filter(|x| **x == Type::Primitive(Primitive::Bool))
            .count();

        let count_i32 = types.len() - count_bool;

        let align = (count_i32 != 0) as usize * std::mem::align_of::<i32>();

        let size = 
              count_bool * std::mem::size_of::<bool>()
            + count_i32  * std::mem::size_of::<i32>();

        let layout = Layout::from_size_align(
            size,
            align.max(std::mem::align_of::<bool>()),
        ).unwrap();

        let mut index = 0;
        
        for (ty, slot) in types.iter().zip(assign.iter_mut()) {
            if let Type::Primitive(Primitive::I32) = ty {
                *slot = index;
                index += std::mem::size_of::<i32>();
            }
        }

        for (ty, slot) in types.iter().zip(assign.iter_mut()) {
            if let Type::Primitive(Primitive::Bool) = ty {
                *slot = index;
                index += std::mem::size_of::<bool>();
            }
        }

        Self {
            types,
            assign,
            frame: StackFrame::new(layout)
        }
    }

    pub fn get<T>(&self, reg: usize) -> T where Self: Get<T> {
        Get::get(self, reg)
    }
    
    pub fn set<T>(&mut self, reg: usize, value: T) where Self: Get<T> {
        *self.get_mut(reg) = value;
    }
    
    pub fn copy(&mut self, from: usize, to: usize) {
        let fty = &self.types[from];
        let tty = &self.types[to];
        assert_eq!(fty, tty);

        let from = self.assign[from];
        let to = self.assign[to];

        let ptr = self.frame.ptr();

        let size = match fty {
            Type::Primitive(Primitive::Bool) => std::mem::size_of::<bool>(),
            Type::Primitive(Primitive::I32) => std::mem::size_of::<i32>(),
            _ => unreachable!()
        };

        unsafe {
            ptr.add(from).copy_to(ptr.add(to), size);
        }
    }
}

pub trait Get<T> {
    fn get(&self, reg: usize) -> T;
    
    fn get_mut(&mut self, reg: usize) -> &mut T;
}

impl Get<bool> for Variables<'_> {
    fn get(&self, reg: usize) -> bool {
        let ty = &self.types[reg];

        assert_eq!(*ty, Type::Primitive(Primitive::Bool));

        let index = self.assign[reg];

        unsafe {
            std::ptr::read(self.frame.ptr().add(index).cast())
        }
    }
    
    fn get_mut(&mut self, reg: usize) -> &mut bool {
        let ty = &self.types[reg];

        assert_eq!(*ty, Type::Primitive(Primitive::Bool));

        let index = self.assign[reg];

        unsafe {
            &mut *self.frame.ptr().add(index).cast()
        }
    }
}

impl Get<i32> for Variables<'_> {
    fn get(&self, reg: usize) -> i32 {
        let ty = &self.types[reg];

        assert_eq!(*ty, Type::Primitive(Primitive::I32));

        let index = self.assign[reg];

        unsafe {
            std::ptr::read(self.frame.ptr().add(index).cast())
        }
    }
    
    fn get_mut(&mut self, reg: usize) -> &mut i32 {
        let ty = &self.types[reg];

        assert_eq!(*ty, Type::Primitive(Primitive::I32));

        let index = self.assign[reg];

        unsafe {
            &mut *self.frame.ptr().add(index).cast()
        }
    }
}
