use krajc::EngineResource;
use wgpu::*;

use crate::{
    engine_runtime::schedule_manager::system_params::system_resource::EngineResource,
    generate_state_struct, InstanceBufferType, Lateinit, UniformBufferType,
};

use super::{
    buffer_manager::managed_buffer::ManagedBufferInstanceHandle,
    builtin_materials::{
        light_material::material::{LightMaterial, LightUniform},
        texture_material::material::TextureMaterial,
    },
    camera::camera::{CameraController, CameraUniform, Projection},
    texture::texture::Texture,
};

pub mod input;
pub mod new;
pub mod render;
pub mod resize;
pub mod update;
pub mod window;

type S = &'static str;

#[derive(Default, EngineResource)]
pub struct RenderManagerResource {
    pub adapter: Lateinit<Adapter>,
    pub surface: Lateinit<Surface>,
    pub device: Lateinit<Device>,
    pub queue: Lateinit<Queue>,
    pub config: Lateinit<SurfaceConfiguration>,
    pub size: Lateinit<winit::dpi::PhysicalSize<u32>>,

    pub texture: Lateinit<Texture>,
    pub depth_texture: Lateinit<Texture>,

    pub window: Lateinit<winit::window::Window>,

    pub light_instance_buffer: Lateinit<ManagedBufferInstanceHandle<InstanceBufferType>>,
    pub texture_instance_buffer: Lateinit<ManagedBufferInstanceHandle<InstanceBufferType>>,

    pub projection: Lateinit<Projection>,
    pub camera_controller: Lateinit<CameraController>,

    pub camera_uniform: Lateinit<CameraUniform>,
    pub camera_buffer: Lateinit<ManagedBufferInstanceHandle<UniformBufferType>>,

    pub light_uniform: Lateinit<LightUniform>,
    pub light_buffer: Lateinit<ManagedBufferInstanceHandle<UniformBufferType>>,

    pub clear_color: Lateinit<Color>,

    pub texture_material: Lateinit<TextureMaterial>,
    pub light_material: Lateinit<LightMaterial>,
}
impl Clone for RenderManagerResource {
    fn clone(&self) -> Self {
        Self::default()
    }
}
