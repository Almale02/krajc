pub mod generic_state_manager;
use std::{
    any::Any,
    fmt::{Debug, Write},
};

use self::generic_state_manager::GenericStateManager;
use dyn_clone::DynClone;
use mopa::mopafy;

pub trait AnyClone: Any + DynClone + mopa::Any {}
mopafy!(AnyClone);

pub struct EngineStateManager {
    pub generic: GenericStateManager,
}

pub struct GenericEngineStateManagerValue {
    pub value: Box<dyn Any>,
    pub ref_count: u128,
    pub ref_mut_count: u128,
}
impl Clone for GenericEngineStateManagerValue {
    fn clone(&self) -> Self {
        Self {
            value: Box::new(0_u8),
            ref_count: 0,
            ref_mut_count: 0,
        }
    }
}
impl GenericEngineStateManagerValue {
    pub fn new(value: Box<dyn Any>) -> Self {
        Self {
            value,
            ref_count: 0,
            ref_mut_count: 0,
        }
    }
}
impl Debug for GenericEngineStateManagerValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(
            format!(
                "ref count {}, ref mut count {}",
                self.ref_count, self.ref_mut_count,
            )
            .as_str(),
        )
    }
}
impl std::fmt::Display for GenericEngineStateManagerValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(
            format!(
                "ref count {}, ref mut count {}",
                self.ref_count, self.ref_mut_count,
            )
            .as_str(),
        )
    }
}

dyn_clone::clone_trait_object!(AnyClone);
