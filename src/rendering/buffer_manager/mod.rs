use std::any::TypeId;
use std::ops::DerefMut;

use crate::engine_runtime::schedule_manager::system_params::system_resource::EngineResource;
use crate::BufferUsages;
use crate::HashMap;
use crate::ManagedBufferGeneric;
use krajc::EngineResource;
use wgpu::Buffer;

pub mod buffer_manager;
pub mod managed_buffer;

#[derive(Default, EngineResource)]
pub struct UniformBufferType {
    instance_handles: HashMap<String, Buffer>,
}
impl ManagedBufferGeneric for UniformBufferType {
    fn buffer_usages() -> wgpu::BufferUsages {
        BufferUsages::UNIFORM | BufferUsages::COPY_DST
    }
    fn label() -> String {
        String::from("uniform buffer")
    }
    fn instance_handles(&mut self) -> &mut std::collections::HashMap<String, Buffer> {
        &mut self.instance_handles
    }
}

#[derive(Default, EngineResource)]
pub struct InstanceBufferType {
    instance_handles: HashMap<String, Buffer>,
}
impl ManagedBufferGeneric for InstanceBufferType {
    fn buffer_usages() -> wgpu::BufferUsages {
        BufferUsages::VERTEX | BufferUsages::COPY_DST
    }
    fn label() -> String {
        String::from("instance_buffer")
    }
    fn instance_handles(
        &mut self,
    ) -> &mut std::collections::HashMap<std::string::String, wgpu::Buffer> {
        &mut self.instance_handles
    }
}

pub fn dupe<'w, T: ?Sized>(value: &'w T) -> &'w mut T {
    unsafe { &mut *((value as *const T) as *mut T) }
}

/// there are times where it is really hard to get the compiler know that you use lifetimes correctly,
/// YOU SHOULD ONLY USE THIS IF YOU KNOW THAT LIFETIMES ARE OK BUT THE COMPILER STILL COMPLAINING, THIS IS LIKE RAW POINTERS FOR LIFETIMES
/// also a general guideline if you need to make a paramater 'static because you couldnt make the compier compile but you know that it is correct then YOU SHOULD MARK YOU SYSTEM UNSAFE
pub unsafe fn dupe_static<T: ?Sized>(value: &T) -> &'static mut T {
    unsafe { &mut *((value as *const T) as *mut T) }
}
