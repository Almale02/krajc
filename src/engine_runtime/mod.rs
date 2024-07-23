use core::panic;
use std::{
    any::{Any, TypeId},
    collections::HashMap,
    marker::PhantomData,
};

use engine_data::{
    data_refs::{EngineDataMut, EngineDataRef},
    data_storage::EngineDataStorage,
};

use crate::{
    ecs::ecs_manager::EcsManager,
    physics::physics_world::PhysicsWorld,
    rendering::{buffer_manager::buffer_manager::BufferManager, managers::RenderManagerResource},
    typed_addr::TypedAddr,
    ENGINE_RUNTIME,
};

use self::schedule_manager::system_params::system_resource::EngineResource;

pub mod engine_data;
pub mod schedule_manager;

pub struct EngineRuntime<'w> {
    pub paralellism: bool,
    pub engine_data: EngineDataStorage<'w>,
    pub static_resource_map: HashMap<TypeId, Box<dyn Any>>,
    pub system_locals: HashMap<&'static str, HashMap<u8, Box<dyn Any>>>,
    pub buffer_manager: BufferManager<'w>,
    pub ecs: EcsManager,
    pub physics: PhysicsWorld,
    _w: PhantomData<&'w ()>,
}

impl<'w: 'static> Default for EngineRuntime<'w> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'w: 'static> EngineRuntime<'w> {
    pub fn new() -> Self {
        Self {
            paralellism: {
                match std::env::var("KRAJC_PARALLELISM") {
                Ok(value) => {match value.as_str() {
                    "true" => true,
                    "false" => false,
                    _ => panic!("invalid value for env variable KRAJC_PARALLELISM, value should be 'true' or 'false'")
                }},
                Err(_) => true/*{true}*/,
            }
            },
            engine_data: EngineDataStorage::new(),
            static_resource_map: Default::default(),
            system_locals: Default::default(),
            buffer_manager: BufferManager::new(),
            ecs: EcsManager::default(),
            physics: PhysicsWorld::default(),
            _w: PhantomData,
        }
    }
    pub fn get_resource_mut<T: EngineResource>(&'w mut self) -> &'w mut T {
        T::get_mut(self)
    }
    pub fn get_resource<T: EngineResource>(&'w mut self) -> &'w T {
        T::get(self)
    }
    pub fn new_data<T: 'static>(&'w mut self, data: T) -> EngineDataRef<'w, T> {
        self.engine_data.create_new(data)
    }
    pub fn new_data_mut<T: 'static>(&'w mut self, data: T) -> EngineDataMut<'w, T> {
        self.engine_data.create_new_mut(data)
    }
}
impl<T> From<&mut T> for TypedAddr<T> {
    fn from(value: &mut T) -> Self {
        TypedAddr::new_with_ref(value)
    }
}
