pub mod prelude {
    pub use crate::*;
}

use std::{
    any::type_name,
    ops::{Deref, DerefMut},
};

#[derive(Debug)]
pub struct Lateinit<T> {
    value: Option<T>,
}
unsafe impl<T> Send for Lateinit<T> {}

impl<T> Lateinit<T> {
    pub fn new(data: T) -> Self {
        Self { value: Some(data) }
    }
    pub fn set(&mut self, value: T) {
        self.value = Some(value);
    }
    pub fn as_option(&self) -> Option<&T> {
        match &self.value {
            Some(val) => Some(val),
            None => None,
        }
    }
    pub fn as_option_mut(&mut self) -> Option<&mut T> {
        match &mut self.value {
            Some(val) => Some(val),
            None => None,
        }
    }
    pub const fn default_const() -> Self {
        Lateinit { value: None }
    }
    pub fn consume(&mut self) -> T {
        match self.value.take() {
            Some(x) => x,
            None => panic!("attempted to consume an uninited Lateinit"),
        }
    }
    pub fn get(&self) -> &T {
        match &self.value {
            Some(value) => value,
            None => {
                panic!(
                    "dereferenced an uninited value with type {:?}",
                    type_name::<T>()
                );
            }
        }
    }
    pub fn get_mut(&mut self) -> &mut T {
        match &mut self.value {
            Some(value) => value,
            None => {
                panic!(
                    "dereferenced an uninited value with type {:?}",
                    type_name::<T>()
                );
            }
        }
    }
}
impl<T> Deref for Lateinit<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        match &self.value {
            Some(value) => value,
            None => {
                panic!(
                    "dereferenced an uninited value with type {:?}",
                    type_name::<T>()
                );
            }
        }
    }
}
impl<T> DerefMut for Lateinit<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match &mut self.value {
            Some(value) => value,
            None => {
                panic!(
                    "dereferenced an uninited value with type {:?}",
                    type_name::<T>()
                );
            }
        }
    }
}
impl<T> Default for Lateinit<T> {
    fn default() -> Self {
        Self { value: None }
    }
}
impl<T: Clone> Clone for Lateinit<T> {
    fn clone(&self) -> Self {
        let value = match &self.value {
            Some(value) => value,
            None => panic!("tried to clone an uninited value"),
        };
        Self {
            value: Some(value.clone()),
        }
    }
}

pub struct ThreadRawPointer<T>(pub *mut T);
impl<T> ThreadRawPointer<T> {
    pub fn new(value: &T) -> ThreadRawPointer<T> {
        ThreadRawPointer((value as *const T) as *mut T)
    }
}

impl<T> Deref for ThreadRawPointer<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe {
            let ptr: &mut T = &mut *self.0;
            ptr
        }
    }
}
impl<T> DerefMut for ThreadRawPointer<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            let ptr: &mut T = &mut *self.0;
            ptr
        }
    }
}

unsafe impl<T> Sync for ThreadRawPointer<T> {}
unsafe impl<T> Send for ThreadRawPointer<T> {}

pub struct Takeable<T> {
    value: Option<T>,
}

impl<T> Takeable<T> {
    pub fn new(value: T) -> Self {
        Takeable { value: Some(value) }
    }

    pub fn take(&mut self) -> Option<T> {
        self.value.take()
    }
}

pub trait AbiTypeId {
    fn uuid() -> &'static str;
}
