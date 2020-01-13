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

#[repr(C)]
struct Array<T, const N: usize>(T, [T; N]);

pub struct ArrayDeque<T, const N: usize> {
    data: MaybeUninit<Array<T, N>>,
    start: usize,
    end: usize,
}

impl<T, const N: usize> ArrayDeque<T, N> {
    pub const fn new() -> Self {
        Self {
            data: MaybeUninit::uninit(),
            start: 0,
            end: 0,
        }
    }
    
    const fn next_slot(slot: usize) -> usize {
        (slot + 1) % (N + 1)
    }

    fn as_ptr(&self) -> *const T {
        self.data.as_ptr() as *const T
    }

    fn as_mut_ptr(&mut self) -> *mut T {
        self.data.as_mut_ptr() as *mut T
    }

    pub const fn is_empty(&self) -> bool {
        self.start == self.end
    }

    pub fn is_full(&self) -> bool {
        if N == 0 {
            true
        } else {
            Self::next_slot(self.start) == self.end
        }
    }

    pub const fn len(&self) -> usize {
        self.end.wrapping_sub(self.start) % N
    }

    pub unsafe fn push_back_unchecked(&mut self, value: T) {
        debug_assert!(!self.is_full());

        let ptr = self.as_mut_ptr();

        ptr.add(self.end).write(value);

        self.end = Self::next_slot(self.end);
    }

    pub fn try_push_back(&mut self, value: T) -> Result<(), T> {
        if self.is_full() {
            Err(value)
        } else {
            unsafe {
                self.push_back_unchecked(value);
                Ok(())
            }
        }
    }

    pub fn push_back(&mut self, value: T) {
        assert!(!self.is_full());

        unsafe {
            self.push_back_unchecked(value)
        }
    }

    pub unsafe fn pop_front_unchecked(&mut self) -> T {
        debug_assert!(!self.is_empty());

        let ptr = self.as_ptr();

        let value = ptr.add(self.start).read();

        self.start = Self::next_slot(self.start);

        value
    }

    pub fn try_pop_front(&mut self) -> Option<T> {
        if self.is_empty() {
            None
        } else {
            unsafe {
                Some(self.pop_front_unchecked())
            }
        }
    }

    pub fn pop_front(&mut self) -> T {
        assert!(!self.is_empty());

        unsafe {
            self.pop_front_unchecked()
        }
    }

    pub unsafe fn front_unchecked(&self) -> &T {
        &*self.as_ptr().add(self.start)
    }

    pub fn front(&self) -> Option<&T> {
        if self.is_empty() {
            None
        } else {
            unsafe {
                Some(self.front_unchecked())
            }
        }
    }

    pub fn iter(&self) -> Iter<'_, T, N> {
        Iter { ptr: self.as_ptr(), start: self.start, end: self.end, lt: PhantomData }
    }
}

use core::marker::PhantomData;

pub struct Iter<'a, T, const N: usize> {
    ptr: *const T,
    start: usize,
    end: usize,
    lt: PhantomData<&'a Array<T, N>>
}

impl<'a, T, const N: usize> Iterator for Iter<'a, T, N> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start == self.end {
            None
        } else {
            unsafe {
                let ptr = self.ptr.add(self.start);

                self.start = ArrayDeque::<T, N>::next_slot(self.start);

                Some(&*ptr)
            }
        }
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        if self.start == self.end {
            return None
        }

        let (start, ovf) = self.start.overflowing_add(n);

        if ovf {
            self.start = self.end;
            return None
        }

        self.start = start % (N + 1);

        self.next()
    }
}

impl<'a, T, const N: usize> IntoIterator for &'a ArrayDeque<T, N> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T, N>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
