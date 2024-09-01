use std::ops::{Deref, DerefMut};

pub struct LazyInitializer<T>(Box<dyn FnOnce() -> T + 'static>);

impl<T> LazyInitializer<T> {
    pub fn new<FN>(callback: FN) -> Self
    where
        FN: FnOnce() -> T + 'static,
    {
        Self(Box::new(callback))
    }

    pub fn init(self) -> T {
        self.0()
    }
}

enum LazyOption<T> {
    Loaded(T),
    Unload(LazyInitializer<T>),
    None,
}

impl<T> std::default::Default for LazyOption<T> {
    fn default() -> Self {
        LazyOption::None
    }
}

impl<T> LazyOption<T> {
    pub fn new<FN>(initializer: FN) -> Self
    where
        FN: FnOnce() -> T + 'static,
    {
        Self::Unload(LazyInitializer::new(initializer))
    }

    pub fn load(&mut self) -> &mut T {
        if let LazyOption::Loaded(value) = self {
            return value;
        }

        if let LazyOption::Unload(initializer) = std::mem::take(self) {
            let value = initializer.init();
            *self = LazyOption::Loaded(value);
        } else {
            panic!("lazy is not Unload");
        }

        if let LazyOption::Loaded(value) = self {
            value
        } else {
            panic!("lazy is not Loaded");
        }
    }
}

pub struct Lazy<T>(*mut LazyOption<T>);

impl<T> Lazy<T> {
    pub fn new<FN>(initializer: FN) -> Self
    where
        FN: FnOnce() -> T + 'static,
    {
        let boxed = Box::new(LazyOption::new(initializer));
        Self(Box::into_raw(boxed))
    }
}

impl<T> Drop for Lazy<T> {
    fn drop(&mut self) {
        unsafe {
            let _ = Box::from_raw(self.0);
        }
    }
}

impl<T> Deref for Lazy<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe {
            let inner = self.0.as_mut().unwrap();
            inner.load()
        }
    }
}

impl<T> DerefMut for Lazy<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            let inner = self.0.as_mut().unwrap();
            inner.load()
        }
    }
}
