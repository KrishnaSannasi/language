use std::alloc::{alloc, dealloc, Layout};

pub struct StackFrame {
    frame: *mut u8,
    layout: Layout,
}

impl StackFrame {
    pub fn new(layout: Layout) -> Self {
        assert_ne!(layout.size(), 0);

        unsafe {
            Self {
                frame: alloc(layout),
                layout
            }
        }
    }

    pub fn ptr(&self) -> *mut u8 {
        self.frame
    }
}

impl Drop for StackFrame {
    fn drop(&mut self) {
        unsafe {
            dealloc(self.frame, self.layout);
        }
    }
}
