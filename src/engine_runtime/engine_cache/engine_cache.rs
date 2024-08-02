use std::{any::Any, collections::HashMap, marker::PhantomData};

use crate::{engine_runtime::EngineRuntime, typed_addr::dupe, FromEngine};

pub struct EngineCache {
    // zero is an invalid id
    pub next_id: u64,
    pub data: HashMap<u64, Box<dyn Any>>,
}

impl EngineCache {
    pub fn new() -> Self {
        Self {
            next_id: 0,
            data: HashMap::new(),
        }
    }
    pub fn new_cache<'cache, T: 'static>(&'cache mut self, data: T) -> (u64, &'cache mut T) {
        let new_id = self.next_id + 1;
        self.next_id += 1;

        self.data.insert(new_id, Box::new(data));

        (
            new_id,
            self.data.get_mut(&new_id).unwrap().downcast_mut().unwrap(),
        )
    }
}

impl Default for EngineCache {
    fn default() -> Self {
        Self::new()
    }
}
pub struct CacheHandle<T> {
    pub cache: &'static mut EngineCache,
    pub id: u64,
    _p: PhantomData<T>,
}

impl<T: 'static> CacheHandle<T> {
    pub fn new(cache: &mut EngineCache) -> Self {
        Self {
            cache: dupe(cache),
            id: 0,
            _p: PhantomData,
        }
    }
    pub fn cache_mut(&mut self, data: impl FnOnce() -> T) -> &mut T {
        match self.cache.data.contains_key(&self.id) {
            true => self
                .cache
                .data
                .get_mut(&self.id)
                .unwrap()
                .downcast_mut()
                .unwrap(),
            false => {
                let (id, data) = self.cache.new_cache(data());
                self.id = id;
                data
            }
        }
    }
    pub fn cache(&mut self, data: impl FnOnce() -> T) -> &T {
        match self.cache.data.contains_key(&self.id) {
            true => self
                .cache
                .data
                .get_mut(&self.id)
                .unwrap()
                .downcast_mut()
                .unwrap(),
            false => {
                dbg!("cache miss");
                let (id, data) = self.cache.new_cache(data());
                self.id = id;
                data
            }
        }
    }
}

impl<T: 'static> FromEngine for CacheHandle<T> {
    fn from_engine(engine: &'static mut EngineRuntime) -> Self {
        Self::new(&mut engine.engine_cache)
    }
}
