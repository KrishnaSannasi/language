#![feature(try_trait)]

use std::mem::{MaybeUninit, ManuallyDrop};
use std::ptr::NonNull;

use std::ops::Try;

const INLINE_CAPACITY_BYTES: usize = std::mem::size_of::<usize>() * 2;

#[repr(C)]
pub struct SmallVec<T> {
    cap: usize,
    len: MaybeUninit<usize>,
    ptr: MaybeUninit<NonNull<T>>,
}

unsafe impl<A: Send> Send for SmallVec<A> {}
unsafe impl<A: Sync> Sync for SmallVec<A> {}

impl<T> Default for SmallVec<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> SmallVec<T> {
    const T_SIZE: usize = std::mem::size_of::<T>();
    
    pub fn new() -> Self {
        assert!(std::mem::align_of::<T>() <= std::mem::align_of::<usize>());

        Self {
            cap: 0,
            len: MaybeUninit::uninit(),
            ptr: MaybeUninit::uninit(),
        }
    }

    #[inline(always)]
    pub fn inline_capacity() -> usize {
        if Self::T_SIZE == 0 {
            usize::max_value()
        } else {
            INLINE_CAPACITY_BYTES / Self::T_SIZE
        }
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.cap.max(Self::inline_capacity())
    }

    #[inline]
    pub fn len(&self) -> usize {
        if self.cap <= Self::inline_capacity() {
            self.cap
        } else {
            unsafe {
                self.len.assume_init()
            }
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn as_ptr(&self) -> *const T {
        unsafe {
            if self.cap <= Self::inline_capacity() {
                (self as *const Self as *const u8)
                    .add(std::mem::size_of::<usize>())
                    .cast()
            } else {
                self.ptr.assume_init().as_ptr()
            }
        }
    }

    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut T {
        unsafe {
            if self.cap <= Self::inline_capacity() {
                (self as *mut Self as *mut u8)
                    .add(std::mem::size_of::<usize>())
                    .cast()
            } else {
                self.ptr.assume_init().as_ptr()
            }
        }
    }

    pub fn as_parts(&self) -> (*const T, usize, usize) {
        unsafe {
            if self.cap <= Self::inline_capacity() {
                let len = self.cap;
                let cap = Self::inline_capacity();
                
                let ptr = (self as *const Self as *const u8)
                    .add(std::mem::size_of::<usize>())
                    .cast::<T>();
                
                (ptr, len, cap)
            } else {
                let len = self.len.assume_init();
                let cap = self.cap;
                let ptr = self.ptr.assume_init().as_ptr();
    
                (ptr, len, cap)
            }
        }
    }

    unsafe fn as_parts_mut(&mut self) -> (*mut T, &mut usize, usize) {
        if self.cap <= Self::inline_capacity() {
            let ptr = (self as *mut Self as *mut u8)
                .add(std::mem::size_of::<usize>())
                .cast::<T>();
            
            let len = &mut self.cap;
            let cap = Self::inline_capacity();
            
            (ptr, len, cap)
        } else {
            let len = &mut *self.len.as_mut_ptr();
            let cap = self.cap;
            let ptr = self.ptr.assume_init().as_ptr();

            (ptr, len, cap)
        }
    }

    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        let (_, len, cap) = self.as_parts();

        let new_cap = len.saturating_add(additional);

        if new_cap > cap {
            unsafe {
                self.reserve_slow(additional, new_cap)
            }
        }
    }

    #[cold]
    unsafe fn reserve_slow(&mut self, additional: usize, new_cap: usize) {
        let (ptr, len, cap) = self.as_parts_mut();
            let len = *len;

            let mut vec = if cap == Self::inline_capacity() {
                let mut vec = ManuallyDrop::new(Vec::<T>::with_capacity(new_cap));

                ptr.copy_to_nonoverlapping(
                        vec.as_mut_ptr(),
                        len
                    );
                
                vec
            } else {
                let mut vec = Vec::from_raw_parts(ptr, len, cap);

                vec.reserve(additional);

                ManuallyDrop::new(vec)
            };

            self.cap = vec.capacity();
            self.len = MaybeUninit::new(len);
            self.ptr = MaybeUninit::new(NonNull::new_unchecked(vec.as_mut_ptr()));
    }

    pub fn push(&mut self, value: T) {
        self.reserve(1);
        
        unsafe {
            let (ptr, len, _) = self.as_parts_mut();

            ptr.add(*len).write(value);
            
            *len += 1;
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        unsafe {
            let (ptr, len, _) = self.as_parts_mut();

            if let Some(l) = len.checked_sub(1) {
                *len = l;

                Some(ptr.add(l).read())
            } else {
                None
            }
        }
    }

    pub fn as_slice(&self) -> &[T] {
        let (ptr, len, _) = self.as_parts();

        unsafe {
            std::slice::from_raw_parts(ptr, len)
        }
    }

    pub fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe {
            let (ptr, len, _) = self.as_parts_mut();

            std::slice::from_raw_parts_mut(ptr, *len)
        }
    }

    pub fn clear(&mut self) {
        self.truncate(0)
    }

    pub fn truncate(&mut self, new_len: usize) {
        unsafe {
            let (ptr, len, _) = self.as_parts_mut();

            assert!(new_len <= *len);

            let diff = *len - new_len;

            let slice = std::slice::from_raw_parts_mut(ptr.add(new_len), diff);

            *len = new_len;

            std::ptr::drop_in_place(slice);
        }
    }

    pub fn shrink_to_fit(&mut self) {
        unsafe {
            let (ptr, len, cap) = self.as_parts_mut();
            let len = *len;

            if cap <= Self::inline_capacity() {
                // if inline, do nothing
            } else if len <= Self::inline_capacity() {
                self.cap = len;

                let inline_ptr = self.as_mut_ptr();

                ptr.copy_to_nonoverlapping(
                    inline_ptr,
                    len
                );

                Vec::from_raw_parts(ptr, 0, cap);
            } else {
                let mut vec = ManuallyDrop::new(Vec::from_raw_parts(ptr, len, cap));

                vec.shrink_to_fit();

                self.cap = vec.capacity();
                self.len = MaybeUninit::new(vec.len());
                self.ptr = MaybeUninit::new(NonNull::new_unchecked(vec.as_mut_ptr()));
            }
        }
    }

    pub fn extend_from_slice(&mut self, slice: &[T])
    where
        T: Clone,
    {
        self.reserve(slice.len());

        unsafe {
            let (ptr, len, _) = self.as_parts_mut();

            let mut ptr = ptr.add(*len);
        
            for item in slice {
                ptr.write(item.clone());

                ptr = ptr.add(1);
                *len += 1;
            }
        }
    }

    pub fn extend_from_slice_copy(&mut self, slice: &[T])
    where
        T: Copy,
    {
        self.reserve(slice.len());

        unsafe {
            let (ptr, len, _) = self.as_parts_mut();
            
            slice.as_ptr()
                .copy_to_nonoverlapping(
                    ptr.add(*len),
                    slice.len()
                );

            *len += slice.len();
        }
    }

    pub fn dedup_by_key<F, K>(&mut self, mut key: F) where F: FnMut(&mut T) -> K, K: PartialEq {
        self.dedup_by(|a, b| key(a) == key(b))
    }

    pub fn dedup_by<F>(&mut self, same_bucket: F) where F: FnMut(&mut T, &mut T) -> bool {
        let len = {
            let (dedup, _) = slice_partition_dedup_by(self.as_mut_slice(), same_bucket);
            dedup.len()
        };

        self.truncate(len);
    }
}

fn slice_partition_dedup_by<T, F>(
    slice: &mut [T], mut same_bucket: F
) -> (&mut [T], &mut [T])
where F: FnMut(&mut T, &mut T) -> bool
{
    let len = slice.len();
        if len <= 1 {
            return (slice, &mut [])
        }

        let ptr = slice.as_mut_ptr();
        let mut next_read: usize = 1;
        let mut next_write: usize = 1;

        unsafe {
            // Avoid bounds checks by using raw pointers.
            while next_read < len {
                let ptr_read = ptr.add(next_read);
                let prev_ptr_write = ptr.add(next_write - 1);
                if !same_bucket(&mut *ptr_read, &mut *prev_ptr_write) {
                    if next_read != next_write {
                        let ptr_write = prev_ptr_write.offset(1);
                        std::mem::swap(&mut *ptr_read, &mut *ptr_write);
                    }
                    next_write += 1;
                }
                next_read += 1;
            }
        }

        slice.split_at_mut(next_write)
}

impl<T: PartialEq> SmallVec<T> {
    pub fn dedup(&mut self) {
        self.dedup_by(|a, b| a == b)
    }
}

impl<T> Drop for SmallVec<T> {
    fn drop(&mut self) {
        unsafe {
            let (ptr, len, cap) = self.as_parts_mut();
            let len = *len;

            std::ptr::drop_in_place(std::slice::from_raw_parts_mut(ptr, len));

            if cap > Self::inline_capacity() {
                Vec::from_raw_parts(ptr, 0, cap);
            }
        }
    }
}

impl<T> std::ops::Deref for SmallVec<T> {
    type Target = [T];

    fn deref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<T> std::ops::DerefMut for SmallVec<T> {
    fn deref_mut(&mut self) -> &mut [T] {
        self.as_mut_slice()
    }
}

impl<T> Extend<T> for SmallVec<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        let iter = iter.into_iter();

        self.reserve(iter.size_hint().0);
        iter.for_each(|value| self.push(value))
    }
}

impl<'a, T: 'a + Clone> Extend<&'a T> for SmallVec<T> {
    fn extend<I: IntoIterator<Item = &'a T>>(&mut self, iter: I) {
        self.extend(iter.into_iter().cloned());
    }
}

use std::ops::{Index, IndexMut};

impl<I, T> Index<I> for SmallVec<T>
where
    [T]: Index<I>,
{
    type Output = <[T] as Index<I>>::Output;

    fn index(&self, index: I) -> &Self::Output {
        self.as_slice().index(index)
    }
}

impl<I, T> IndexMut<I> for SmallVec<T>
where
    [T]: IndexMut<I>,
{
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        self.as_mut_slice().index_mut(index)
    }
}

impl<T, U> std::iter::FromIterator<U> for SmallVec<T>
where
    Self: Extend<U>,
{
    fn from_iter<I: IntoIterator<Item = U>>(iter: I) -> Self {
        let mut vec = Self::new();
        vec.extend(iter);
        vec
    }
}

impl<A: AsRef<[T]>, T> PartialEq<A> for SmallVec<T>
where
    [T]: PartialEq
{
    fn eq(&self, other: &A) -> bool {
        self.as_slice() == other.as_ref()
    }
}

impl<A: AsRef<[T]>, T> PartialOrd<A> for SmallVec<T>
where
    [T]: PartialOrd
{
    fn partial_cmp(&self, other: &A) -> Option<std::cmp::Ordering> {
        self.as_slice().partial_cmp(other.as_ref())
    }
}

impl<T> Eq for SmallVec<T>
where
    [T]: Eq
{}

impl<T> Ord for SmallVec<T>
where
    [T]: Ord
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.as_slice().cmp(other.as_slice())
    }
}

impl<T> AsRef<[T]> for SmallVec<T> {
    fn as_ref(&self) -> &[T] {
        self
    }
}

impl<T> AsMut<[T]> for SmallVec<T> {
    fn as_mut(&mut self) -> &mut [T] {
        self
    }
}

impl<T> std::borrow::Borrow<[T]> for SmallVec<T> {
    fn borrow(&self) -> &[T] {
        self
    }
}

impl<T> std::borrow::BorrowMut<[T]> for SmallVec<T> {
    fn borrow_mut(&mut self) -> &mut [T] {
        self
    }
}

impl<T: Clone> From<&[T]> for SmallVec<T> {
    fn from(slice: &[T]) -> Self {
        let mut vec = SmallVec::new();

        vec.extend_from_slice(slice);

        vec
    }
}

impl<'a, T> IntoIterator for &'a SmallVec<T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

use std::fmt;
impl<T: fmt::Debug> fmt::Debug for SmallVec<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.as_slice().fmt(f)
    }
}

impl<T: Clone> Clone for SmallVec<T> {
    fn clone(&self) -> Self {
        let mut vec = Self::new();
        vec.extend_from_slice(self);
        vec
    }
}

#[repr(C)]
pub struct IntoIter<T> {
    idx: usize,
    vec: ManuallyDrop<SmallVec<T>>,
}

impl<T> IntoIterator for SmallVec<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            idx: 0,
            vec: ManuallyDrop::new(self)
        }
    }
}

impl<T> Drop for IntoIter<T> {
    fn drop(&mut self) {
        unsafe {
            let (ptr, len, cap) = self.vec.as_parts_mut();
            let len = *len;

            std::ptr::drop_in_place(std::slice::from_raw_parts_mut(
                ptr.add(self.idx),
                len - self.idx,
            ));

            if cap > SmallVec::<T>::inline_capacity() {
                Vec::from_raw_parts(ptr, 0, cap);
            }
        }
    }
}

impl<T> ExactSizeIterator for IntoIter<T> {}
impl<T> std::iter::FusedIterator for IntoIter<T> {}
impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let (ptr, len, _) = self.vec.as_parts();
        let idx = self.idx;

        if idx < len {
            unsafe {
                self.idx += 1;
                Some(ptr.add(idx).read())
            }
        } else {
            None
        }
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let (ptr, len, _) = unsafe {
            self.vec.as_parts_mut()
        };

        let len = *len;

        let end = self.idx.saturating_add(n);

        if end < len {
            unsafe {
                std::ptr::drop_in_place(std::slice::from_raw_parts_mut(
                    ptr.add(self.idx),
                    n,
                ));
                self.idx = end;
                Some(ptr.add(end).read())
            }
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.vec.len() - self.idx;

        (len, Some(len))
    }
    
    fn try_fold<B, F, R>(&mut self, mut init: B, mut f: F) -> R
    where
        F: FnMut(B, Self::Item) -> R,
        R: Try<Ok = B>,
    {
        let (ptr, len, _) = self.vec.as_parts();
        
        while self.idx < len {
            unsafe {
                let idx = self.idx;
                self.idx += 1;
                init = f(init, ptr.add(idx).read())?
            }
        }

        R::from_ok(init)
    }
}

#[test]
fn foo() {
    let mut vec = SmallVec::<Box<u8>>::new();

    for i in 0..16 {
        vec.push(Box::new(i));
    }

    drop(vec);

    let mut vec = SmallVec::<Box<u8>>::new();

    for i in 0..50 {
        vec.push(Box::new(i));
    }

    for i in (0..50).rev() {
        assert_eq!(Some(i), vec.pop().as_deref().copied())
    }
    assert_eq!(None, vec.pop());

    let mut vec = SmallVec::<()>::new();

    for _ in 0..17 {
        vec.push(());
    }

    assert_eq!(vec.len(), 17);

    for _ in (0..17).rev() {
        assert_eq!(Some(()), vec.pop())
    }
    assert_eq!(None, vec.pop());
    
    let mut vec = SmallVec::<u8>::new();

    for i in 0..50 {
        vec.push(i);
    }

    for _ in 0..40 {
        vec.pop();
    }

    vec.shrink_to_fit();
}
