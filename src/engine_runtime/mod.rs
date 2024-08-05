use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

use engine_cache::engine_cache::EngineCache;

use crate::{
    ecs::ecs_manager::EcsManager,
    physics::physics_world::PhysicsWorld,
    rendering::{
        asset::AssetManager, buffer_manager::buffer_manager::BufferManager,
        managers::RenderManagerResource,
    },
    typed_addr::{dupe, TypedAddr},
    ENGINE_RUNTIME,
};

use self::schedule_manager::system_params::system_resource::EngineResource;

pub mod engine_cache;
pub mod schedule_manager;
pub mod target_fps;

#[derive(Default)]
pub struct EngineRuntime {
    pub paralellism: bool,
    pub static_resource_map: HashMap<TypeId, usize>,
    pub system_locals: HashMap<&'static str, HashMap<u8, Box<dyn Any>>>,
    pub buffer_manager: BufferManager,
    pub render_resource_manager: AssetManager,
    pub engine_cache: EngineCache,
    pub ecs: EcsManager,
    pub physics: PhysicsWorld,
}

impl EngineRuntime {
    pub fn new() -> Self {
        Self {
            paralellism: { false },
            /*match std::env::var("KRAJC_PARALLELISM") {
                Ok(value) => {match value.as_str() {
                    "true" => true,
                    "false" => false,
                    _ => panic!("invalid value for env variable KRAJC_PARALLELISM, value should be 'true' or 'false'")
                }},
                Err(_) => true/*{true}*/,
            }*/
            ..Default::default()
        }
    }
    pub fn init() -> &'static mut Self {
        let mgr = Box::new(Self::new());
        let leaked = Box::leak(mgr);
        let raw = leaked as *mut _;
        let render_states_addr = raw as usize;
        unsafe { ENGINE_RUNTIME.addr = render_states_addr }

        leaked
    }
    pub fn get_resource_without_insert<T>(&self) -> Option<&'static mut T> {
        let x = self.static_resource_map.get(&TypeId::of::<T>());

        x?;
        Some(TypedAddr::new(*x.unwrap()).get())
    }
    pub fn get_resource_mut<T: EngineResource>(&mut self) -> &'static mut T {
        T::get_mut(dupe(self))
    }
    pub fn get_resource<T: EngineResource>(&mut self) -> &'static T {
        T::get(dupe(self))
    }
}
#[derive(Default)]
pub struct StateNames {
    pub render_mgr: RenderManagerResource,
}
impl<T> From<&mut T> for TypedAddr<T> {
    fn from(value: &mut T) -> Self {
        TypedAddr::new_with_ref(value)
    }
}
