use crate::engine_runtime::{EngineRuntime};
use crate::rendering::buffer_manager::managed_buffer::ManagedBuffer;
use crate::rendering::managers::RenderManagerResource;

use crate::{addr_ptr_to_ref_mut};
use std::any::TypeId;
use std::collections::HashMap;


use super::managed_buffer::ManagedBufferGeneric;

pub struct BufferManager {
    pub engine: &'static mut EngineRuntime,
    buffers: HashMap<TypeId, ManagedBuffer>,
}
impl BufferManager {
    pub fn new() -> Self {
        Self {
            engine: addr_ptr_to_ref_mut!(0, EngineRuntime, "buff manager new", true),
            buffers: Default::default(),
        }
    }
    pub fn register_new_buffer<T: ManagedBufferGeneric + Default + 'static>(&mut self) {
        self.buffers.insert(
            TypeId::of::<T>(),
            Box::leak(Box::new(T::default()))
                .get_managed_buffer(self.engine.get_resource::<RenderManagerResource>()),
        );
    }
    pub fn get_buffer<T: ManagedBufferGeneric + 'static>(&'static self) -> &'static ManagedBuffer {
        self.buffers.get(&TypeId::of::<T>()).unwrap()
    }
    pub fn get_buffer_mut<T: ManagedBufferGeneric + 'static>(
        &'static mut self,
    ) -> &'static mut ManagedBuffer {
        self.buffers.get_mut(&TypeId::of::<T>()).unwrap()
    }
}
