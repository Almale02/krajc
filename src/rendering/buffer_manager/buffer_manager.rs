use crate::engine_runtime::EngineRuntime;
use crate::rendering::buffer_manager::managed_buffer::ManagedBuffer;
use crate::rendering::managers::RenderManagerResource;
use crate::{
    engine_runtime::schedule_manager::system_params::system_resource::EngineResource, Lateinit,
};

use crate::addr_ptr_to_ref_mut;
use std::any::TypeId;
use std::collections::HashMap;

use super::managed_buffer::{clone_ref, ManagedBufferGeneric};
use super::{dupe, dupe_static};

pub struct BufferManager<'w> {
    pub engine: Lateinit<&'w mut EngineRuntime<'w>>,
    buffers: HashMap<TypeId, ManagedBuffer<'w>>,
}
impl<'w: 'static> BufferManager<'w> {
    pub fn new() -> Self {
        Self {
            engine: Lateinit::default(),
            buffers: Default::default(),
        }
    }
    pub fn register_new_buffer<T: ManagedBufferGeneric + Default + EngineResource + 'static>(
        &'w mut self,
    ) {
        let render: &mut _ = dupe(*self.engine).get_resource_mut::<RenderManagerResource>();
        dupe(*self.engine).buffer_manager.buffers.insert(
            TypeId::of::<T>(),
            dupe(*self.engine)
                .get_resource_mut::<T>()
                .get_managed_buffer(render),
        );
    }
    pub fn get_buffer<T: ManagedBufferGeneric + 'static>(&'w self) -> &'w ManagedBuffer {
        self.buffers.get(&TypeId::of::<T>()).unwrap()
    }
    pub fn get_buffer_mut<T: ManagedBufferGeneric + 'static>(
        &'w mut self,
    ) -> &'w mut ManagedBuffer {
        self.buffers.get_mut(&TypeId::of::<T>()).unwrap()
    }
}
