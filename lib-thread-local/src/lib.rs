#![allow(clippy::transmute_ptr_to_ptr)]

use parking_lot::Mutex;
use std::collections::HashMap;
use std::thread::{current, ThreadId};

pub struct ThreadLocal<T> {
    inner: Mutex<HashMap<ThreadId, Box<T>>>,
}
pub struct LazyThreadLocal<T, F = fn() -> T> {
    inner: ThreadLocal<T>,
    init: F,
}

impl<T> Default for ThreadLocal<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> ThreadLocal<T> {
    pub fn new() -> Self {
        Self {
            inner: Mutex::default(),
        }
    }

    pub fn get(&self) -> Option<&T> {
        let inner = self.inner.lock();
        let value = inner.get(&current().id())?;

        unsafe { Some(std::mem::transmute::<&T, &T>(value)) }
    }

    pub fn get_or(&self, value: T) -> &T {
        self.get_or_else(move || value)
    }

    pub fn get_or_else(&self, value: impl FnOnce() -> T) -> &T {
        let mut inner = self.inner.lock();
        let value = inner
            .entry(current().id())
            .or_insert_with(move || Box::new(value()));

        unsafe { std::mem::transmute::<&T, &T>(value) }
    }
}

impl<T, F: Fn() -> T> LazyThreadLocal<T, F> {
    pub fn new(init: F) -> Self {
        Self {
            init,
            inner: ThreadLocal::new(),
        }
    }

    pub fn get(&self) -> Option<&T> {
        self.inner.get()
    }

    pub fn force_init(&self) -> &T
    where
        F: Fn() -> T,
    {
        self.inner.get_or_else(&self.init)
    }
}

impl<T, F: Fn() -> T> std::ops::Deref for LazyThreadLocal<T, F> {
    type Target = T;

    fn deref(&self) -> &T {
        self.force_init()
    }
}
