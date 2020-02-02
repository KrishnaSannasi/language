use std::borrow::Borrow;
use std::cell::Cell;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::mem::MaybeUninit;

use parking_lot::RwLock;

pub struct Cache<T> {
    #[allow(clippy::vec_box)]
    data: RwLock<HashSet<Item<T>>>,
}

impl<T: Hash + Eq> Default for Cache<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Hash + Eq> Cache<T> {
    pub fn new() -> Self {
        Self {
            data: RwLock::default(),
        }
    }

    pub fn get<Q>(&self, value: &T) -> Option<&T>
    where
        T: Borrow<Q>,
        Q: Hash + Eq,
    {
        let data = self.data.read();

        let data = data.get(value)?;

        unsafe { Some(&*data.ptr) }
    }

    #[inline(never)]
    #[allow(clippy::mut_from_ref)]
    pub fn insert(&self, value: T) -> &T {
        let mut data = self.data.write();

        let mark = Cell::new(true);
        let value = Inserter {
            item: Cell::new(MaybeUninit::new(value)),
        };

        let output = data.get_or_insert_with(&value, |value| unsafe {
            mark.set(false);
            Item {
                own: PhantomData,
                ptr: Box::into_raw(Box::new(
                    value.item.replace(MaybeUninit::uninit()).assume_init(),
                )),
            }
        });

        if mark.get() {
            unsafe {
                value.item.as_ptr().cast::<T>().drop_in_place();
            }
        }

        unsafe { &*output.ptr }
    }
}

struct Item<T> {
    ptr: *mut T,
    own: PhantomData<T>,
}

#[repr(transparent)]
struct Inserter<T> {
    item: Cell<MaybeUninit<T>>,
}

impl<T> Borrow<Inserter<T>> for Item<T> {
    fn borrow(&self) -> &Inserter<T> {
        unsafe { &*self.ptr.cast::<Inserter<T>>() }
    }
}

impl<T> Borrow<T> for Item<T> {
    fn borrow(&self) -> &T {
        unsafe { &*self.ptr }
    }
}

impl<T: Eq> Eq for Item<T> {}
impl<T: PartialEq> PartialEq for Item<T> {
    fn eq(&self, other: &Self) -> bool {
        unsafe { *self.ptr == *other.ptr }
    }
}

impl<T: Hash> Hash for Item<T> {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        unsafe {
            (*self.ptr).hash(hasher);
        }
    }
}

impl<T: Eq> Eq for Inserter<T> {}
impl<T: PartialEq> PartialEq for Inserter<T> {
    fn eq(&self, other: &Self) -> bool {
        unsafe { *self.item.as_ptr().cast::<T>() == *other.item.as_ptr().cast::<T>() }
    }
}

impl<T: Hash> Hash for Inserter<T> {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        unsafe {
            (*self.item.as_ptr().cast::<T>()).hash(hasher);
        }
    }
}

unsafe impl<#[may_dangle] T> Drop for Item<T> {
    fn drop(&mut self) {
        unsafe { self.ptr.drop_in_place() }
    }
}

#[test]
fn miri() {
    let c = Cache::new();

    assert!(c.get(&10).is_none());

    let a = c.insert(10);
    let b = c.insert(10);

    assert_eq!(c.get(&10), Some(&10));

    assert_eq!(a, b);
    assert_eq!(a as *const i32, b as *const i32);
}
