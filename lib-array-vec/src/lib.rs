#![feature(const_generics, dropck_eyepatch)]
// #![no_std]

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
    size_info: SizeInfo<N>,
}

#[derive(Clone, Copy)]
pub struct SizeInfo<const N: usize> {
    start: usize,
    end: usize,
}

impl<const N: usize> SizeInfo<N> {
    const CAPACITY: usize = N + 1;

    const fn len(&self) -> usize {
        let slow = {
            let (len, ovf) = self.end.overflowing_sub(self.start);

            let nlen = Self::CAPACITY
                .wrapping_sub(self.start)
                .wrapping_add(self.end);

            [len, nlen][ovf as usize]
        };

        let fast = self.end.wrapping_sub(self.start) & N;

        [slow, fast][Self::CAPACITY.is_power_of_two() as usize]
    }

    const fn is_empty(&self) -> bool {
        self.start == self.end
    }

    pub fn inc(var: &mut usize) {
        Self::add(var, 1)
    }

    pub fn add(var: &mut usize, n: usize) {
        *var = (*var + n) % Self::CAPACITY;
    }
}

impl<T, const N: usize> ArrayDeque<T, N> {
    pub const fn new() -> Self {
        Self {
            data: MaybeUninit::uninit(),
            size_info: SizeInfo { start: 0, end: 0 },
        }
    }

    fn as_ptr(&self) -> *const T {
        self.data.as_ptr() as *const T
    }

    fn as_mut_ptr(&mut self) -> *mut T {
        self.data.as_mut_ptr() as *mut T
    }

    pub const fn is_empty(&self) -> bool {
        self.size_info.is_empty()
    }

    pub const fn is_full(&self) -> bool {
        self.len() == N
    }

    pub const fn len(&self) -> usize {
        self.size_info.len()
    }

    pub unsafe fn push_back_unchecked(&mut self, value: T) {
        debug_assert!(!self.is_full());

        let ptr = self.as_mut_ptr();

        ptr.add(self.size_info.end).write(value);

        SizeInfo::<N>::inc(&mut self.size_info.end);
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
        assert!(!self.is_full(), "Tried to push into a full queue");

        unsafe { self.push_back_unchecked(value) }
    }

    pub unsafe fn pop_front_unchecked(&mut self) -> T {
        debug_assert!(!self.is_empty());

        let ptr = self.as_ptr();

        let value = ptr.add(self.size_info.start).read();

        SizeInfo::<N>::inc(&mut self.size_info.start);

        value
    }

    pub fn try_pop_front(&mut self) -> Option<T> {
        if self.is_empty() {
            None
        } else {
            unsafe { Some(self.pop_front_unchecked()) }
        }
    }

    pub fn pop_front(&mut self) -> T {
        assert!(!self.is_empty());

        unsafe { self.pop_front_unchecked() }
    }

    pub unsafe fn front_unchecked(&self) -> &T {
        &*self.as_ptr().add(self.size_info.start)
    }

    pub fn front(&self) -> Option<&T> {
        if self.is_empty() {
            None
        } else {
            unsafe { Some(self.front_unchecked()) }
        }
    }

    pub fn iter(&self) -> Iter<'_, T, N> {
        Iter {
            ptr: self.as_ptr(),
            size_info: self.size_info,
            lt: PhantomData,
        }
    }
}

use core::marker::PhantomData;

pub struct Iter<'a, T, const N: usize> {
    ptr: *const T,
    size_info: SizeInfo<N>,
    lt: PhantomData<&'a Array<T, N>>,
}

impl<'a, T, const N: usize> ExactSizeIterator for Iter<'a, T, N> {}
impl<'a, T, const N: usize> core::iter::FusedIterator for Iter<'a, T, N> {}
impl<'a, T, const N: usize> Iterator for Iter<'a, T, N> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if N == 0 || self.size_info.is_empty() {
            None
        } else {
            unsafe {
                let ptr = self.ptr.add(self.size_info.start);
                SizeInfo::<N>::inc(&mut self.size_info.start);
                Some(&*ptr)
            }
        }
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        if N == 0 || n >= self.size_info.len() {
            None
        } else {
            SizeInfo::<N>::add(&mut self.size_info.start, n);

            unsafe {
                let ptr = self.ptr.add(self.size_info.start);
                SizeInfo::<N>::inc(&mut self.size_info.start);
                Some(&*ptr)
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        if N == 0 {
            (0, Some(0))
        } else {
            let len = self.size_info.len();
            (len, Some(len))
        }
    }
}

pub struct IterMut<'a, T, const N: usize> {
    ptr: *mut T,
    size_info: SizeInfo<N>,
    lt: PhantomData<&'a mut Array<T, N>>,
}

impl<'a, T, const N: usize> ExactSizeIterator for IterMut<'a, T, N> {}
impl<'a, T, const N: usize> core::iter::FusedIterator for IterMut<'a, T, N> {}
impl<'a, T, const N: usize> Iterator for IterMut<'a, T, N> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        if N == 0 || self.size_info.is_empty() {
            None
        } else {
            unsafe {
                let ptr = self.ptr.add(self.size_info.start);
                SizeInfo::<N>::inc(&mut self.size_info.start);
                Some(&mut *ptr)
            }
        }
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        if N == 0 || n >= self.size_info.len() {
            None
        } else {
            SizeInfo::<N>::add(&mut self.size_info.start, n);

            unsafe {
                let ptr = self.ptr.add(self.size_info.start);
                SizeInfo::<N>::inc(&mut self.size_info.start);
                Some(&mut *ptr)
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        if N == 0 {
            (0, Some(0))
        } else {
            let len = self.size_info.len();
            (len, Some(len))
        }
    }
}

impl<'a, T, const N: usize> IntoIterator for &'a ArrayDeque<T, N> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T, N>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

unsafe impl<#[may_dangle] T, const N: usize> Drop for ArrayDeque<T, N> {
    fn drop(&mut self) {
        unsafe {
            let SizeInfo { start, end } = self.size_info;

            if let Some(len) = end.checked_sub(start) {
                core::ptr::drop_in_place(core::slice::from_raw_parts_mut(
                    self.as_mut_ptr().add(start),
                    len,
                ))
            } else {
                let ptr = self.as_mut_ptr();
                core::ptr::drop_in_place(core::slice::from_raw_parts_mut(ptr, start));

                core::ptr::drop_in_place(core::slice::from_raw_parts_mut(
                    ptr.add(end),
                    SizeInfo::<N>::CAPACITY - end,
                ));
            }
        }
    }
}

#[test]
fn foo() {
    let mut a = ArrayDeque::<_, 15>::new();
    let mut v = Vec::new();

    for i in 0..14 {
        a.push_back(i);
        v.push(i);

        for _ in 0..1000 {
            let len = a.len();
            assert_eq!(len, v.len());
            a.push_back(0);
            a.pop_front();
            assert_eq!(a.len(), len);
        }
    }

    let mut a = ArrayDeque::<_, 20>::new();
    let mut v = Vec::new();

    for i in 0..19 {
        a.push_back(i);
        v.push(i);

        for _ in 0..1000 {
            let len = a.len();
            assert_eq!(len, v.len());
            a.push_back(0);
            a.pop_front();
            assert_eq!(a.len(), len);
        }
    }
}

#[test]
fn iter() {
    let mut a = ArrayDeque::<_, 15>::new();

    for i in 0..15 {
        a.push_back(i)
    }

    for _ in 0..5 {
        let i = a.pop_front();
        a.push_back(i);
    }

    for _ in 0..15 {
        a.pop_front();
    }

    for i in 0..15 {
        a.push_back(i)
    }

    for n in 1..10 {
        dbg!();
        dbg!();
        dbg!(n);
        dbg!();
        for (i, &v) in a.iter().step_by(n).enumerate() {
            dbg!(v);
            assert_eq!(i * n, v);
        }
    }
}
