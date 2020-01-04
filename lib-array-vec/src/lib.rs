#![feature(const_generics, dropck_eyepatch)]
#![no_std]

use core::mem::MaybeUninit;

pub struct ArrayVec<T, const N: usize> {
    data: MaybeUninit<[T; N]>,
    len: usize,
}

impl<T, const N: usize> Default for ArrayVec<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const N: usize> ArrayVec<T, N> {
    pub const fn new() -> Self {
        Self {
            data: MaybeUninit::uninit(),
            len: 0,
        }
    }

    pub fn as_ptr(&self) -> *const T {
        self.data.as_ptr() as *const T
    }

    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.data.as_mut_ptr() as *mut T
    }

    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub const fn is_full(&self) -> bool {
        self.len == N
    }

    pub const fn len(&self) -> usize {
        self.len
    }

    pub fn as_slice(&self) -> &[T] {
        unsafe { core::slice::from_raw_parts(self.as_ptr(), self.len) }
    }

    pub fn as_slice_mut(&mut self) -> &mut [T] {
        unsafe { core::slice::from_raw_parts_mut(self.as_mut_ptr(), self.len) }
    }

    ///
    /// # Safety
    ///
    /// The first `len` elements of the `ArrayVec` must be initialized
    pub unsafe fn set_len(&mut self, len: usize) {
        debug_assert!(len < N);
        self.len = len;
    }

    ///
    /// # Safety
    ///
    /// The `ArrayVec` must not be full
    pub unsafe fn push_unchecked(&mut self, value: T) -> &mut T {
        debug_assert!(
            N > 0 && N > self.len,
            "Tried to push into a full `ArrayVec`"
        );

        let ptr = self.as_mut_ptr().add(self.len);

        ptr.write(value);

        self.len += 1;

        &mut *ptr
    }

    pub fn try_push(&mut self, value: T) -> Result<&mut T, T> {
        if N == 0 || N == self.len {
            Err(value)
        } else {
            unsafe { Ok(self.push_unchecked(value)) }
        }
    }

    pub fn push(&mut self, value: T) -> &mut T {
        assert!(
            N > 0 && N > self.len,
            "Tried to push into a full `ArrayVec`"
        );

        unsafe { self.push_unchecked(value) }
    }

    ///
    /// # Safety
    ///
    /// The `ArrayVec` must not be empty
    pub unsafe fn pop_unchecked(&mut self) -> T {
        self.len -= 1;

        self.as_ptr().add(self.len).read()
    }

    pub fn try_pop(&mut self) -> Option<T> {
        if N == 0 || self.len == 0 {
            None
        } else {
            unsafe { Some(self.pop_unchecked()) }
        }
    }

    pub fn pop(&mut self) -> T {
        assert!(N == 0 || self.len > 0, "Tried to pop an empty `ArrayVec`");

        unsafe { self.pop_unchecked() }
    }
}

unsafe impl<#[may_dangle] T, const N: usize> Drop for ArrayVec<T, N> {
    fn drop(&mut self) {
        unsafe { core::ptr::drop_in_place(self.as_slice_mut()) }
    }
}
