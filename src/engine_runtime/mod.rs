use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

use crate::{
    rendering::{buffer_manager::buffer_manager::BufferManager, managers::RenderManagerResource},
    typed_addr::TypedAddr,
    ENGINE_RUNTIME,
};

use self::{
    engine_state_manager::{generic_state_manager::GenericStateManager, EngineStateManager},
    schedule_manager::system_params::system_resource::EngineResource,
};

pub mod engine_state_manager;
pub mod schedule_manager;

pub struct EngineRuntime {
    pub state: EngineStateManager,
    pub static_resource_map: HashMap<TypeId, usize>,
    pub system_locals: HashMap<&'static str, HashMap<u8, Box<dyn Any>>>,
    pub buffer_manager: BufferManager,
}

impl Default for EngineRuntime {
    fn default() -> Self {
        Self::new()
    }
}

impl EngineRuntime {
    pub fn new() -> Self {
        Self {
            state: EngineStateManager {
                generic: GenericStateManager::new(),
            },
            static_resource_map: Default::default(),
            system_locals: Default::default(),
            buffer_manager: BufferManager::new(),
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
    pub fn get_resource<T: EngineResource>(&mut self) -> &'static mut T {
        let x = self.static_resource_map.get(&TypeId::of::<T>());

        let address;

        if x.is_none() {
            let addr: TypedAddr<_> = T::init(self).into();
            address = addr.addr;
        } else {
            address = *x.unwrap();
        }

        TypedAddr::new(address).get()
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
