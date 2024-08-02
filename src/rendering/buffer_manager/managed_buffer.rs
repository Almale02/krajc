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

pub trait ManagedBufferGeneric {
    fn instance_handles(&mut self) -> &mut HashMap<String, (&'static [u8], Buffer)>;
    fn buffer_usages() -> BufferUsages;
    fn label() -> String;
    fn get_managed_buffer(
        &'static mut self,
        render: &'static mut RenderManagerResource,
    ) -> ManagedBuffer {
        ManagedBuffer {
            instance_handles: Self::instance_handles(self),
            buffer_usages: Self::buffer_usages(),
            label: Self::label(),
            render,
        }
    }
}
pub struct ManagedBuffer {
    pub instance_handles: &'static mut HashMap<String, (&'static [u8], Buffer)>,
    pub buffer_usages: BufferUsages,
    pub label: String,
    render: &'static mut RenderManagerResource,
}

pub struct ManagedBufferInstanceHandle<T> {
    pub id: String,
    pub engine: &'static mut EngineRuntime,
    _p: PhantomData<T>,
}

impl<T: ManagedBufferGeneric + 'static> ManagedBufferInstanceHandle<T> {
    pub fn new(id: String) -> Self {
        Self {
            id,
            _p: PhantomData,
            engine: unsafe { ENGINE_RUNTIME.get() },
        }
    }
    pub fn new_with_size(id: String, size: u64) -> Self {
        let instance = Self::new(id);
        let engine = unsafe { ENGINE_RUNTIME.get() };
        engine
            .buffer_manager
            .get_buffer_type_mut::<T>()
            .create_managed_buffer_size(instance.clone(), size);

        instance
    }
    pub fn new_with_init<A: NoUninit>(id: String, data: A) -> Self {
        let instance = Self::new(id);
        let engine = unsafe { ENGINE_RUNTIME.get() };
        engine
            .buffer_manager
            .get_buffer_type_mut::<T>()
            .create_managed_buffer_init(instance.clone(), data);

        instance
    }
    pub fn new_with_init_vec<A: NoUninit>(id: String, data: Vec<A>) -> Self {
        let instance = Self::new(id);
        let engine = unsafe { ENGINE_RUNTIME.get() };
        engine
            .buffer_manager
            .get_buffer_type_mut::<T>()
            .create_managed_buffer_init_vec(instance.clone(), data);

        instance
    }
    pub fn get_buffer(&self) -> &Buffer {
        self.clone()
            .engine
            .buffer_manager
            .get_buffer_type::<T>()
            .get_buffer(self.clone())
    }
    pub fn set_data<A: NoUninit>(&self, data: A) {
        self.clone()
            .engine
            .buffer_manager
            .get_buffer_type_mut::<T>()
            .update_buffer(self.clone(), data);
    }
    pub fn set_data_vec<A: NoUninit>(&self, data: Vec<A>) {
        self.clone()
            .engine
            .buffer_manager
            .get_buffer_type_mut::<T>()
            .update_buffer_vec(self.clone(), data);
    }
}
impl<T: ManagedBufferGeneric> Clone for ManagedBufferInstanceHandle<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            engine: addr_ptr_to_ref_mut!(
                (self.engine as *const EngineRuntime) as usize,
                EngineRuntime,
                "a"
            ),
            _p: PhantomData,
        }
    }
}

impl<T: ManagedBufferGeneric> From<SystemParam> for ManagedBufferInstanceHandle<T> {
    fn from(value: SystemParam) -> Self {
        let id = format!("{}:{}", value.fn_name, value.position);

        ManagedBufferInstanceHandle::<T> {
            id,
            engine: value.engine,
            _p: PhantomData,
        }
    }
}

impl ManagedBuffer {
    pub fn get_buffer<T>(&self, buffer_instance: ManagedBufferInstanceHandle<T>) -> &Buffer {
        &self.instance_handles.get(&buffer_instance.id).unwrap().1
    }
    pub fn update_buffer<T: ManagedBufferGeneric + 'static, A: NoUninit>(
        &mut self,
        buffer_instance: ManagedBufferInstanceHandle<T>,
        data: A,
    ) {
        let buffer = buffer_instance.get_buffer();
        self.render
            .queue
            .write_buffer(buffer, 0, bytemuck::cast_slice(&[data]));
    }
    pub fn update_buffer_vec<T: ManagedBufferGeneric + 'static, A: NoUninit>(
        &mut self,
        buffer_instance: ManagedBufferInstanceHandle<T>,
        data: Vec<A>,
    ) {
        let buffer = buffer_instance.get_buffer();
        self.render
            .queue
            .write_buffer(buffer, 0, bytemuck::cast_slice(&*data));
    }
    pub fn create_managed_buffer<T: ManagedBufferGeneric + 'static>(
        &mut self,
        buffer_instance: ManagedBufferInstanceHandle<T>,
    ) -> &Buffer {
        let buffer = self
            .render
            .device
            .create_buffer_init(&BufferInitDescriptor {
                label: Some(self.label.clone().as_str()),
                contents: &[0],
                usage: self.buffer_usages,
            });
        self.instance_handles
            .insert(buffer_instance.id.clone(), (&[0], buffer));

        addr_ptr_to_ref_mut!(
            ((buffer_instance.clone().get_buffer() as *const _) as usize),
            Buffer,
            "a"
        )
    }

    pub fn create_managed_buffer_size<T: ManagedBufferGeneric + 'static>(
        &mut self,
        buffer_instance: ManagedBufferInstanceHandle<T>,
        size: u64,
    ) -> &Buffer {
        if size % wgpu::COPY_BUFFER_ALIGNMENT != 0 {
            panic!("buffer size wasnt multiple of copy buffer alignment which is 4 u64")
        }
        let buffer = self.render.device.create_buffer(&BufferDescriptor {
            label: Some("create_managed_buffer_size"),
            size,
            usage: self.buffer_usages,
            mapped_at_creation: false,
        });
        self.instance_handles.insert(
            buffer_instance.id.clone(),
            (bytemuck::cast_slice(Box::leak(Box::new([0]))), buffer),
        );

        addr_ptr_to_ref_mut!(
            ((buffer_instance.clone().get_buffer() as *const _) as usize),
            Buffer,
            "a"
        )
    }

    pub fn create_managed_buffer_init<T: ManagedBufferGeneric + 'static, A: NoUninit>(
        &mut self,
        buffer_instance: ManagedBufferInstanceHandle<T>,
        data: A,
    ) -> &Buffer {
        let buffer = self
            .render
            .device
            .create_buffer_init(&BufferInitDescriptor {
                label: Some(self.label.clone().as_str()),
                contents: bytemuck::cast_slice(&[data]),
                usage: self.buffer_usages,
            });
        self.instance_handles.insert(
            buffer_instance.id.clone(),
            (bytemuck::cast_slice(Box::leak(Box::new([data]))), buffer),
        );

        addr_ptr_to_ref_mut!(
            ((buffer_instance.clone().get_buffer() as *const _) as usize),
            Buffer,
            "a"
        )
    }
    pub fn create_managed_buffer_init_vec<T: ManagedBufferGeneric + 'static, A: NoUninit>(
        &mut self,
        buffer_instance: ManagedBufferInstanceHandle<T>,
        data: Vec<A>,
    ) -> &Buffer {
        let buffer = self
            .render
            .device
            .create_buffer_init(&BufferInitDescriptor {
                label: Some(self.label.clone().as_str()),
                contents: bytemuck::cast_slice(&data),
                usage: self.buffer_usages,
            });
        self.instance_handles.insert(
            buffer_instance.id.clone(),
            (bytemuck::cast_slice(&[0]), buffer),
        );

        addr_ptr_to_ref_mut!(
            ((buffer_instance.clone().get_buffer() as *const _) as usize),
            Buffer,
            "a"
        )
    }
}
