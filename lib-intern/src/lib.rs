#![feature(hash_set_entry)]

use std::alloc::{alloc, dealloc, handle_alloc_error, Layout};
use std::collections::HashSet;
use std::marker::PhantomData;
use std::ptr::NonNull;

use parking_lot::RwLock;

const USIZE_SIZE: usize = std::mem::size_of::<usize>();
const USIZE_ALIGN: usize = std::mem::align_of::<usize>();

#[repr(C)]
struct StrInner {
    len: usize,
    data: [u8],
}

#[derive(Eq)]
struct OwnStr(NonNull<()>);

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Str<'a> {
    ptr: NonNull<()>,
    mark: PhantomData<&'a StrInner>,
}

#[derive(Default)]
pub struct Intern {
    inner: RwLock<HashSet<OwnStr>>,
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

impl Str<'_> {
    pub fn as_str(&self) -> &str {
        self
    }
}

impl AsRef<str> for Str<'_> {
    fn as_ref(&self) -> &str {
        self
    }
}

impl<'a: 'b, 'b> AsRef<Str<'b>> for Str<'a> {
    fn as_ref(&self) -> &Str<'b> {
        self
    }
}

impl std::borrow::Borrow<str> for Str<'_> {
    fn borrow(&self) -> &str {
        self
    }
}

impl std::fmt::Debug for Str<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Debug::fmt(self.as_str(), f)
    }
}

impl std::fmt::Display for Str<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(self.as_str(), f)
    }
}

impl std::ops::Deref for Str<'_> {
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

impl Intern {
    pub fn new() -> Self {
        Self {
            inner: RwLock::new(HashSet::new()),
        }
    }

    pub fn len(&self) -> usize {
        self.inner.read().len()
    }

    pub fn insert(&self, s: &(impl AsRef<str> + ?Sized)) -> Str<'_> {
        self.insert_inner(s.as_ref())
    }

    fn insert_inner(&self, s: &str) -> Str<'_> {
        let inner = self.inner.read();
        if let Some(own_str) = inner.get(s) {
            Str {
                ptr: own_str.0,
                mark: PhantomData,
            }
        } else {
            drop(inner);
            self.insert_slow(s)
        }
    }

    #[cold]
    fn insert_slow(&self, s: &str) -> Str<'_> {
        let mut inner = self.inner.write();

        let own_str = inner.get_or_insert_with(s, |s: &str| unsafe {
            let layout = Layout::from_size_align_unchecked(s.len() + USIZE_SIZE, USIZE_ALIGN);
            let ptr = alloc(layout);

            let str_repr = match NonNull::new(ptr as *mut ()) {
                Some(ptr) => ptr,
                None => handle_alloc_error(layout),
            };

            ptr.cast::<usize>().write(s.len());

            ptr.add(USIZE_SIZE).copy_from(s.as_ptr(), s.len());

            OwnStr(str_repr)
        });

        Str {
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
