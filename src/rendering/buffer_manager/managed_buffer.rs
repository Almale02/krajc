use crate::engine_runtime::schedule_manager::system_params::system_param::SystemParam;
use crate::engine_runtime::EngineRuntime;
use crate::rendering::managers::RenderManagerResource;
use crate::{addr_ptr_to_ref_mut, ENGINE_RUNTIME};
use bytemuck::NoUninit;
use core::panic;
use std::collections::HashMap;
use std::marker::PhantomData;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{Buffer, BufferDescriptor, BufferUsages};

use super::dupe;

pub trait ManagedBufferGeneric {
    fn instance_handles(&mut self) -> &mut HashMap<String, Buffer>;
    fn buffer_usages() -> BufferUsages;
    fn label() -> String;
    fn get_managed_buffer<'w>(
        &'w mut self,
        render: &'w mut RenderManagerResource<'w>,
    ) -> ManagedBuffer<'w> {
        ManagedBuffer {
            instance_handles: self.instance_handles(),
            buffer_usages: Self::buffer_usages(),
            label: Self::label(),
            render,
        }
    }
}

/// ONLY SHOULD BE USED FOR CLONING REFERENCES IF YOU KNOW THAT THE STRUCT YOU ARE CLONING A REFERENCE TO WILL OUTLIVE EVERYTHING
pub unsafe fn clone_ref<T: ?Sized>(value: &T) -> &'static mut T {
    unsafe { &mut *((value as *const T) as *mut T) }
}
pub struct ManagedBuffer<'w> {
    pub instance_handles: &'w mut HashMap<String, Buffer>,
    pub buffer_usages: BufferUsages,
    pub label: String,
    render: &'w mut RenderManagerResource<'w>,
}

pub struct ManagedBufferInstanceHandle<'w, T> {
    pub id: String,
    pub engine: &'w mut EngineRuntime<'w>,
    _p: PhantomData<T>,
}

impl<'w, T: ManagedBufferGeneric + 'static> ManagedBufferInstanceHandle<'w, T> {
    pub fn new(id: String, engine: &'w mut EngineRuntime<'w>) -> Self {
        Self {
            id,
            _p: PhantomData,
            engine,
        }
    }
    pub fn clone(&'w self) -> Self {
        Self {
            id: self.id.clone(),
            engine: dupe(self.engine),
            _p: PhantomData,
        }
    }
    pub fn new_with_size(id: String, size: u64, engine: &'w mut EngineRuntime<'w>) -> Self {
        let instance = ManagedBufferInstanceHandle::<'w>::new(id, dupe(engine));
        let a = dupe(engine)
            .buffer_manager
            .get_buffer_mut::<T>()
            .create_managed_buffer_size::<T>(instance.id.clone(), size);

        instance
    }
    pub fn new_with_init<A: NoUninit>(
        id: String,
        data: A,
        engine: &'w mut EngineRuntime<'w>,
    ) -> Self {
        let instance = Self::new(id, engine);
        let engine = unsafe { ENGINE_RUNTIME.get() };
        engine
            .buffer_manager
            .get_buffer_mut::<T>()
            .create_managed_buffer_init::<T, A>(instance.id.clone(), data);

        instance
    }
    pub fn new_with_init_vec<A: NoUninit>(
        id: String,
        data: Vec<A>,
        engine: &'w mut EngineRuntime<'w>,
    ) -> Self {
        let instance = Self::new(id, engine);
        let engine = unsafe { ENGINE_RUNTIME.get() };
        engine
            .buffer_manager
            .get_buffer_mut::<T>()
            .create_managed_buffer_init_vec::<T, A>(instance.id.clone(), data);

        instance
    }
    pub fn get_buffer(&'w self) -> &'w Buffer {
        self.clone()
            .engine
            .buffer_manager
            .get_buffer::<T>()
            .get_buffer(self)
    }
    pub fn set_data<A: NoUninit>(&'w self, data: A) {
        self.clone()
            .engine
            .buffer_manager
            .get_buffer_mut::<T>()
            .update_buffer(self, data);
    }
    pub fn set_data_vec<A: NoUninit>(&'w self, data: Vec<A>) {
        self.clone()
            .engine
            .buffer_manager
            .get_buffer_mut::<T>()
            .update_buffer_vec(self, data);
    }
}

impl<'w, T: ManagedBufferGeneric> From<SystemParam<'w>> for ManagedBufferInstanceHandle<'w, T> {
    fn from(value: SystemParam<'w>) -> Self {
        let id = format!("{}:{}", value.fn_name, value.position);

        ManagedBufferInstanceHandle::<T> {
            id,
            engine: value.engine,
            _p: PhantomData,
        }
    }
}

impl<'w> ManagedBuffer<'w> {
    pub fn get_buffer<T>(
        &'w self,
        buffer_instance: &'w ManagedBufferInstanceHandle<'w, T>,
    ) -> &'w Buffer {
        &self.instance_handles.get(&buffer_instance.id).unwrap()
    }
    pub fn update_buffer<T: ManagedBufferGeneric + 'w + 'static, A: NoUninit>(
        &mut self,
        buffer_instance: &'w ManagedBufferInstanceHandle<'w, T>,
        data: A,
    ) {
        let buffer = buffer_instance.get_buffer();
        self.render
            .queue
            .write_buffer(buffer, 0, bytemuck::cast_slice(&[data]));
    }
    pub fn update_buffer_vec<T: ManagedBufferGeneric + 'w + 'static, A: NoUninit>(
        &mut self,
        buffer_instance: &'w ManagedBufferInstanceHandle<'w, T>,
        data: Vec<A>,
    ) {
        let buffer = buffer_instance.get_buffer();
        self.render
            .queue
            .write_buffer(buffer, 0, bytemuck::cast_slice(&*data));
    }

    pub fn create_managed_buffer<T: ManagedBufferGeneric + 'static>(
        &mut self,
        //buffer_instance: &'w ManagedBufferInstanceHandle<'w, T>,
        buffer_instance: String,
    ) /*-> &'w Buffer*/
    {
        let buffer = self
            .render
            .device
            .create_buffer_init(&BufferInitDescriptor {
                label: Some(self.label.clone().as_str()),
                contents: &[0],
                usage: self.buffer_usages,
            });
        self.instance_handles.insert(buffer_instance, buffer);

        //buffer_instance.get_buffer()
    }

    pub fn create_managed_buffer_size<T: ManagedBufferGeneric + 'static>(
        &mut self,
        //buffer_instance: &'w ManagedBufferInstanceHandle<'w, T>,
        buffer_instance: String,
        size: u64,
    ) /*-> &'w Buffer*/
    {
        if size % wgpu::COPY_BUFFER_ALIGNMENT != 0 {
            panic!("buffer size wasnt multiple of copy buffer alignment which is 4 u64")
        }
        let buffer = self.render.device.create_buffer(&BufferDescriptor {
            label: Some("create_managed_buffer_size"),
            size,
            usage: self.buffer_usages,
            mapped_at_creation: false,
        });
        self.instance_handles.insert(buffer_instance, buffer);

        //buffer_instance.get_buffer()
    }

    pub fn create_managed_buffer_init<T: ManagedBufferGeneric + 'static, A: NoUninit>(
        &mut self,
        //buffer_instance: &'w ManagedBufferInstanceHandle<'w, T>,
        buffer_instance: String,
        data: A,
    ) /* -> &'w Buffer*/
    {
        let buffer = self
            .render
            .device
            .create_buffer_init(&BufferInitDescriptor {
                label: Some(self.label.clone().as_str()),
                contents: bytemuck::cast_slice(&[data]),
                usage: self.buffer_usages,
            });
        self.instance_handles.insert(buffer_instance, buffer);

        //buffer_instance.get_buffer()
    }
    pub fn create_managed_buffer_init_vec<T: ManagedBufferGeneric + 'static, A: NoUninit>(
        &mut self,
        buffer_instance: String,
        data: Vec<A>,
    ) /* -> &'w Buffer*/
    {
        let buffer = self
            .render
            .device
            .create_buffer_init(&BufferInitDescriptor {
                label: Some(self.label.clone().as_str()),
                contents: bytemuck::cast_slice(&data),
                usage: self.buffer_usages,
            });
        self.instance_handles.insert(buffer_instance, buffer);

        //buffer_instance.get_buffer()
    }
}
