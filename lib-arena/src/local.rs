use core::cell::{Cell, UnsafeCell};
use std::alloc::{alloc, dealloc, handle_alloc_error, Layout};

pub struct LocalUniqueArena<T, const N: usize> {
    current: Cell<*mut T>,
    len: Cell<usize>,
    #[allow(clippy::vec_box)]
    data: UnsafeCell<Vec<Box<[T; N]>>>,
}

unsafe impl<T: Send, const N: usize> Send for LocalUniqueArena<T, N> {}
impl<T, const N: usize> !Sync for LocalUniqueArena<T, N> {}

impl<T, const N: usize> Default for LocalUniqueArena<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const N: usize> LocalUniqueArena<T, N> {
    pub fn new() -> Self {
        assert!(N != 0, "Cannot use empty slabs for an arena");
        assert!(
            std::mem::size_of::<T>() != 0,
            "Cannot use zero-sized types in arenas"
        );

        let current = unsafe { alloc(Layout::new::<[T; N]>()) };

        if current.is_null() {
            handle_alloc_error(Layout::new::<[T; N]>())
        }

        let current = Cell::new(current as *mut T);

        Self {
            current,
            len: Cell::new(0),
            data: UnsafeCell::default(),
        }
    }

    #[inline(never)]
    #[allow(clippy::mut_from_ref)]
    pub fn alloc(&self, value: T) -> &mut T {
        unsafe {
            if self.len.get() == N {
                self.alloc_slow(value)
            } else {
                let len = self.len.get();
                let current = self.current.get().add(len);

                current.write(value);

                self.len.set(len + 1);

                &mut *current
            }
        }
    }

    #[cold]
    #[inline(never)]
    #[allow(clippy::mut_from_ref)]
    unsafe fn alloc_slow(&self, value: T) -> &mut T {
        let current = self.current.get();
        let slab = Box::from_raw(current as *mut [T; N]);
        self.len.set(0);
        (*self.data.get()).push(slab);

        let slab = alloc(Layout::new::<[T; N]>());

        if slab.is_null() {
            handle_alloc_error(Layout::new::<[T; N]>())
        }

        self.len.set(1);
        let current = slab as *mut T;

        self.current.set(current);
        current.write(value);

        &mut *current
    }
}

unsafe impl<#[may_dangle] T, const N: usize> Drop for LocalUniqueArena<T, N> {
    fn drop(&mut self) {
        unsafe {
            std::ptr::drop_in_place(std::slice::from_raw_parts_mut(
                self.current.get(),
                self.len.get(),
            ));

            dealloc(self.current.get() as _, Layout::new::<[T; N]>());
        }
    }
}

pub struct LocalSharedArena<T, const N: usize> {
    inner: LocalUniqueArena<T, N>,
}

impl<T, const N: usize> !Sync for LocalSharedArena<T, N> {}

impl<T, const N: usize> Default for LocalSharedArena<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const N: usize> LocalSharedArena<T, N> {
    pub fn new() -> Self {
        Self {
            inner: LocalUniqueArena::new(),
        }
    }

    #[inline(never)]
    #[allow(clippy::mut_from_ref)]
    pub fn alloc(&self, value: T) -> &T {
        self.inner.alloc(value)
    }
}
