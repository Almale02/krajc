use crate::BufferUsages;
use crate::HashMap;
use crate::ManagedBufferGeneric;
use wgpu::Buffer;

pub mod buffer_manager;
pub mod managed_buffer;

#[derive(Default)]
pub struct UniformBufferType {
    instance_handles: HashMap<String, (&'static [u8], Buffer)>,
}
impl ManagedBufferGeneric for UniformBufferType {
    fn buffer_usages() -> wgpu::BufferUsages {
        BufferUsages::UNIFORM | BufferUsages::COPY_DST
    }
    fn label() -> String {
        String::from("uniform buffer")
    }
    fn instance_handles(
        &mut self,
    ) -> &mut std::collections::HashMap<String, (&'static [u8], Buffer)> {
        &mut self.instance_handles
    }
}

#[derive(Default)]
pub struct StorageBufferType {
    instance_handles: HashMap<String, (&'static [u8], Buffer)>,
}
impl ManagedBufferGeneric for StorageBufferType {
    fn buffer_usages() -> wgpu::BufferUsages {
        BufferUsages::STORAGE | BufferUsages::COPY_DST
    }
    fn label() -> String {
        String::from("uniform buffer")
    }
    fn instance_handles(
        &mut self,
    ) -> &mut std::collections::HashMap<String, (&'static [u8], Buffer)> {
        &mut self.instance_handles
    }
}

#[derive(Default)]
pub struct InstanceBufferType {
    instance_handles: HashMap<String, (&'static [u8], Buffer)>,
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
    ) -> &mut std::collections::HashMap<String, (&'static [u8], Buffer)> {
        &mut self.instance_handles
    }
}
