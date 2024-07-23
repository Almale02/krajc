use std::{any::Any, collections::HashMap, marker::PhantomData};

use super::data_refs::{EngineDataMut, EngineDataRef};

pub struct EngineDataStorage<'w> {
    pub data_maps: HashMap<u128, Box<dyn Any>>,
    next_id: u128,
    _w: PhantomData<&'w ()>,
}

impl<'w> EngineDataStorage<'w> {
    pub fn new() -> Self {
        Self {
            data_maps: HashMap::default(),
            next_id: 0,
            _w: PhantomData,
        }
    }

    pub fn get_data<T: 'static>(&'w self, id: &u128) -> &'w T {
        self.data_maps.get(id).unwrap().downcast_ref().unwrap()
    }
    pub fn get_data_mut<T: 'static>(&'w mut self, id: &u128) -> &'w mut T {
        self.data_maps.get_mut(id).unwrap().downcast_mut().unwrap()
    }
    pub fn free_data(&mut self, id: &u128) {
        self.data_maps.remove(id);
    }
    pub fn create_new<T: 'static>(&'w mut self, data: T) -> EngineDataRef<'w, T> {
        let curr_id = self.next_id;
        self.next_id += 1;
        self.data_maps.insert(curr_id, Box::new(data));

        EngineDataRef::new(curr_id, self)
    }
    pub fn create_new_mut<T: 'static>(&'w mut self, data: T) -> EngineDataMut<'w, T> {
        let curr_id = self.next_id;
        self.next_id += 1;
        self.data_maps.insert(curr_id, Box::new(data));

        EngineDataMut::new(curr_id, self)
    }
}
