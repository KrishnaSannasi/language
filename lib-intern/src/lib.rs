#![feature(hash_set_entry)]

use std::alloc::{alloc, dealloc, handle_alloc_error, Layout};
use std::collections::HashSet;
use std::marker::PhantomData;
use std::ptr::NonNull;

use parking_lot::RwLock;
use std::cell::UnsafeCell;

const USIZE_SIZE: usize = std::mem::size_of::<usize>();
const USIZE_ALIGN: usize = std::mem::align_of::<usize>();

const _: () = [()][!(USIZE_SIZE > 1) as usize];

#[repr(C)]
struct StrInner {
    len: usize,
    data: [u8],
}

#[derive(Eq)]
struct OwnStr(NonNull<()>);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Intern;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Data;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Dynamic;

pub type InternStr<'a> = Str<'a, Intern>;
pub type DataStr<'a> = Str<'a, Data>;

#[derive(Clone, Copy)]
pub struct Str<'a, T = Dynamic> {
    ptr: NonNull<()>,
    mark: PhantomData<(&'a StrInner, T)>,
}

impl<T> Eq for Str<'_, T> where Self: PartialEq {}

impl PartialEq<Str<'_, Intern>> for Str<'_, Intern> {
    fn eq(&self, other: &Str<'_, Intern>) -> bool {
        self.ptr == other.ptr
    }
}

impl PartialEq<Str<'_, Data>> for Str<'_, Data> {
    fn eq(&self, other: &Str<'_, Data>) -> bool {
        self.as_str() == other.as_str()
    }
}

impl PartialEq<Str<'_, Dynamic>> for Str<'_, Dynamic> {
    fn eq(&self, other: &Str<'_, Dynamic>) -> bool {
        if self.ptr == other.ptr {
            return true;
        }

        let s_ptr = self.ptr.as_ptr() as usize;
        let o_ptr = other.ptr.as_ptr() as usize;

        match ((s_ptr & 1) == 0, (o_ptr & 1) == 0) {
            (true, true) => {
                let s = DataStr {
                    ptr: self.ptr,
                    mark: PhantomData,
                };
                let o = DataStr {
                    ptr: other.ptr,
                    mark: PhantomData,
                };

                s == o
            }
            (false, false) | (false, true) | (true, false) => false,
        }
    }
}

impl PartialOrd<Str<'_, Intern>> for Str<'_, Intern> {
    fn partial_cmp(&self, other: &Str<'_, Intern>) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialOrd<Str<'_, Data>> for Str<'_, Data> {
    fn partial_cmp(&self, other: &Str<'_, Data>) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Str<'_, Intern> {
    fn cmp(&self, other: &Str<'_, Intern>) -> std::cmp::Ordering {
        self.ptr.cmp(&other.ptr)
    }
}

impl Ord for Str<'_, Data> {
    fn cmp(&self, other: &Str<'_, Data>) -> std::cmp::Ordering {
        self.as_str().cmp(other.as_str())
    }
}

impl PartialOrd<Str<'_, Dynamic>> for Str<'_, Dynamic> {
    fn partial_cmp(&self, other: &Str<'_, Dynamic>) -> Option<std::cmp::Ordering> {
        let s_ptr = self.ptr.as_ptr() as usize;
        let o_ptr = other.ptr.as_ptr() as usize;

        match ((s_ptr & 1) == 0, (o_ptr & 1) == 0) {
            (true, true) => {
                let s = DataStr {
                    ptr: self.ptr,
                    mark: PhantomData,
                };
                let o = DataStr {
                    ptr: other.ptr,
                    mark: PhantomData,
                };

                s.partial_cmp(&o)
            }
            (false, false) => {
                let s = InternStr {
                    ptr: self.ptr,
                    mark: PhantomData,
                };
                let o = InternStr {
                    ptr: other.ptr,
                    mark: PhantomData,
                };

                s.partial_cmp(&o)
            }
            (false, true) | (true, false) => None,
        }
    }
}

use std::hash::{Hash, Hasher};

impl Hash for Str<'_, Intern> {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        self.ptr.hash(hasher)
    }
}

impl Hash for Str<'_, Data> {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        self.as_str().hash(hasher)
    }
}

impl Hash for Str<'_, Dynamic> {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        let s_ptr = self.ptr.as_ptr() as usize;

        if (s_ptr & 1) == 0 {
            DataStr {
                ptr: self.ptr,
                mark: PhantomData,
            }
            .hash(hasher)
        } else {
            self.ptr.hash(hasher)
        }
    }
}

#[derive(Default)]
pub struct Interner {
    inner: RwLock<HashSet<OwnStr>>,
}

#[derive(Default)]
pub struct Store {
    inner: UnsafeCell<Vec<OwnStr>>,
}

impl OwnStr {
    fn as_str(&self) -> &str {
        unsafe {
            let ptr = self.0.as_ptr();

            let len = ptr.cast::<usize>().read();

            std::str::from_utf8_unchecked(std::slice::from_raw_parts(
                ptr.cast::<u8>().add(USIZE_SIZE),
                len,
            ))
        }
    }
}

impl PartialEq for OwnStr {
    fn eq(&self, _other: &OwnStr) -> bool {
        unimplemented!()
    }
}

impl std::hash::Hash for OwnStr {
    fn hash<H: std::hash::Hasher>(&self, hasher: &mut H) {
        self.as_str().hash(hasher)
    }
}

impl std::borrow::Borrow<str> for OwnStr {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl<'a> From<Str<'a, Data>> for Str<'a, Dynamic> {
    fn from(s: Str<'a, Data>) -> Self {
        Self {
            ptr: s.ptr,
            mark: PhantomData,
        }
    }
}

impl<'a> From<Str<'a, Intern>> for Str<'a, Dynamic> {
    fn from(s: Str<'a, Intern>) -> Self {
        let ptr = s.ptr.as_ptr() as usize;
        let ptr = ptr | 1;
        let ptr = unsafe { NonNull::new_unchecked(ptr as *mut ()) };
        Self {
            ptr,
            mark: PhantomData,
        }
    }
}

impl<T> Str<'_, T> {
    pub fn as_str(&self) -> &str {
        self
    }
}

impl<T> AsRef<str> for Str<'_, T> {
    fn as_ref(&self) -> &str {
        self
    }
}

impl<'a: 'b, 'b, T> AsRef<Str<'b, T>> for Str<'a, T> {
    fn as_ref(&self) -> &Str<'b, T> {
        self
    }
}

impl<T> std::borrow::Borrow<str> for Str<'_, T> {
    fn borrow(&self) -> &str {
        self
    }
}

impl<T> std::fmt::Debug for Str<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Debug::fmt(self.as_str(), f)
    }
}

impl<T> std::fmt::Display for Str<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.as_str(), f)
    }
}

impl<T> std::ops::Deref for Str<'_, T> {
    type Target = str;

    fn deref(&self) -> &str {
        unsafe {
            let ptr = self.ptr.as_ptr();

            let len = ptr.cast::<usize>().read();

            std::str::from_utf8_unchecked(std::slice::from_raw_parts(
                ptr.cast::<u8>().add(USIZE_SIZE),
                len,
            ))
        }
    }
}

impl From<&str> for OwnStr {
    fn from(s: &str) -> OwnStr {
        unsafe {
            let layout = Layout::from_size_align_unchecked(s.len() + USIZE_SIZE, USIZE_ALIGN);
            let ptr = alloc(layout);

            let str_repr = match NonNull::new(ptr as *mut ()) {
                Some(ptr) => ptr,
                None => handle_alloc_error(layout),
            };

            ptr.cast::<usize>().write(s.len());

            ptr.add(USIZE_SIZE).copy_from(s.as_ptr(), s.len());

            OwnStr(str_repr)
        }
    }
}

impl Store {
    pub const fn new() -> Self {
        Self {
            inner: UnsafeCell::new(Vec::new()),
        }
    }

    pub fn insert(&self, s: &(impl AsRef<str> + ?Sized)) -> DataStr<'_> {
        self.insert_inner(s.as_ref())
    }

    fn insert_inner(&self, s: &str) -> DataStr<'_> {
        let inner = unsafe { &mut *self.inner.get() };

        let own_str = OwnStr::from(s);

        let ret = DataStr {
            ptr: own_str.0,
            mark: PhantomData,
        };

        inner.push(own_str);

        ret
    }
}

#[allow(clippy::len_without_is_empty)]
impl Interner {
    pub fn new() -> Self {
        Self {
            inner: RwLock::new(HashSet::new()),
        }
    }

    pub fn len(&self) -> usize {
        self.inner.read().len()
    }

    pub fn insert(&self, s: &(impl AsRef<str> + ?Sized)) -> InternStr<'_> {
        self.insert_inner(s.as_ref())
    }

    fn insert_inner(&self, s: &str) -> InternStr<'_> {
        let inner = self.inner.read();
        if let Some(own_str) = inner.get(s) {
            InternStr {
                ptr: own_str.0,
                mark: PhantomData,
            }
        } else {
            drop(inner);
            self.insert_slow(s)
        }
    }

    #[cold]
    fn insert_slow(&self, s: &str) -> InternStr<'_> {
        let mut inner = self.inner.write();

        let own_str = inner.get_or_insert_with(s, |s: &str| s.into());

        InternStr {
            ptr: own_str.0,
            mark: PhantomData,
        }
    }
}

impl Drop for OwnStr {
    fn drop(&mut self) {
        unsafe {
            let ptr = self.0.as_ptr();
            let len = ptr.cast::<usize>().read();
            let layout = Layout::from_size_align_unchecked(len + USIZE_SIZE, USIZE_ALIGN);
            dealloc(ptr.cast::<u8>(), layout);
        }
    }
}
