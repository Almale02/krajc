use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

use engine_cache::engine_cache::EngineCache;
use schedule_manager::{
    schedule::{Schedule, ScheduleRunnable},
    system_params::system_param::{SystemParalellFilter, SystemParam},
};

use crate::{
    ecs::ecs_manager::EcsManager,
    rendering::{
        asset::AssetManager, buffer_manager::buffer_manager::BufferManager,
        managers::RenderManagerResource,
    },
    typed_addr::{dupe, TypedAddr},
    AbiTypeId, ENGINE_RUNTIME,
};

use self::schedule_manager::system_params::system_resource::EngineResource;

pub mod engine_cache;
pub mod input;
pub mod schedule_manager;
pub mod target_fps;

#[derive(Default)]
#[repr(C)]
pub struct EngineRuntime {
    pub paralellism: bool,
    pub test: u32,
    pub static_resource_map: HashMap<&'static str, usize>,
    pub system_locals: HashMap<String, HashMap<u8, Box<dyn Any>>>,
    pub system_param_filters: HashMap<String, Vec<Box<dyn SystemParalellFilter>>>,
    pub buffer_manager: BufferManager,
    pub asset_manager: AssetManager,
    pub engine_cache: EngineCache,
    pub ecs: EcsManager,
    pub test_map: HashMap<i32, i32>,
}

unsafe impl Send for EngineRuntime {}

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
    pub fn get_resource_without_insert<T: AbiTypeId>(&self) -> Option<&'static mut T> {
        let x = self.static_resource_map.get(T::uuid());

        x?;
        Some(TypedAddr::new(*x.unwrap()).get())
    }
    pub fn get_resource_mut<T: EngineResource>(&mut self) -> &'static mut T {
        T::get_mut(dupe(self))
    }
    pub fn get_resource<T: EngineResource>(&mut self) -> &'static T {
        T::get(dupe(self))
    }
    pub fn get_resource_no_init<T: EngineResource>(&self) -> &'static T {
        T::get(dupe(self))
    }
    pub fn register_system<T: Schedule>(&mut self, system: impl ScheduleRunnable + 'static) {
        println!("affrwfoijwegfjrj d system with name: {}", system.name());
        self.get_resource_mut::<T>().register(system);
    }
    pub fn register_system_dyn<T: Schedule>(
        &mut self,
        mut system: impl ScheduleRunnable + 'static,
    ) {
        println!("registered dynamically with name: {}", &system.name());
        system.run(dupe(self));
        self.get_resource_mut::<T>().register_dyn(Box::new(system));
    }
    pub fn register_system_box<T: Schedule>(&mut self, mut system: Box<dyn ScheduleRunnable>) {
        println!("registered dynamically with name: {}", &system.name());
        system.run(dupe(self));
        self.get_resource_mut::<T>().register_dyn(system);
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
