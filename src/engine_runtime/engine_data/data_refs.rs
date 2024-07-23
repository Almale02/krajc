use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use crate::{
    engine_runtime::EngineRuntime,
    rendering::buffer_manager::{dupe, dupe_static},
};

use super::data_storage::EngineDataStorage;

pub struct EngineDataRef<'w, T> {
    pub id: u128,
    pub engine: &'w mut EngineDataStorage<'w>,
    _pw: PhantomData<&'w ()>,
    _pt: PhantomData<T>,
}

impl<'w, T> EngineDataRef<'w, T> {
    pub fn new(id: u128, engine: &'w mut EngineDataStorage<'w>) -> Self {
        Self {
            id,
            engine,
            _pw: PhantomData,
            _pt: PhantomData,
        }
    }
}

impl<'w, T: 'w> Deref for EngineDataRef<'w, T> {
    type Target = T;
    fn deref(&self) -> &'w Self::Target {
        todo!()
    }
}
impl<'w, T> Drop for EngineDataRef<'w, T> {
    fn drop(&mut self) {
        self.engine.free_data(&self.id);
    }
}

pub struct EngineDataMut<'w, T> {
    pub id: u128,
    pub engine: &'w mut EngineDataStorage<'w>,
    _pw: PhantomData<&'w ()>,
    _pt: PhantomData<T>,
}

impl<'w, T> EngineDataMut<'w, T> {
    pub fn new(id: u128, engine: &'w mut EngineDataStorage<'w>) -> Self {
        Self {
            id,
            engine,
            _pw: PhantomData,
            _pt: PhantomData,
        }
    }
}

impl<'w, T: 'static> EngineDataMut<'w, T> {
    pub fn get(&'w self) -> &'w T {
        self.engine.get_data(&self.id)
    }
}

impl<'w, T: 'static> EngineDataMut<'w, T> {
    pub fn get_mut(&'w mut self) -> &'w mut T {
        self.engine.get_data_mut(&self.id)
    }
}
impl<'w, T> Drop for EngineDataMut<'w, T> {
    fn drop(&mut self) {
        self.engine.free_data(&self.id);
    }
}
