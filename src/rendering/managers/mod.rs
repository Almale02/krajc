use krajc_macros::EngineResource;
use new::TextState;
use wgpu::*;

use crate::{InstanceBufferType, Lateinit, UniformBufferType};

use super::{
    buffer_manager::{managed_buffer::ManagedBufferInstanceHandle, StorageBufferType},
    camera::camera::{CameraController, CameraUniform, Projection},
    draw_pass::DrawPass,
    lights::{PointLightUniform, SpotLightUniform},
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
pub struct TargetFps(pub f32);

#[derive(Default, EngineResource)]
pub struct RenderManagerResource {
    pub adapter: Lateinit<Adapter>,
    pub surface: Lateinit<Surface>,
    pub device: Lateinit<Device>,
    pub queue: Lateinit<Queue>,
    pub config: Lateinit<SurfaceConfiguration>,
    pub size: Lateinit<winit::dpi::PhysicalSize<u32>>,

    pub depth_texture: Lateinit<Texture>,

    pub window: Lateinit<winit::window::Window>,

    pub light_instance_buffer: Lateinit<ManagedBufferInstanceHandle<InstanceBufferType>>,
    pub texture_instance_buffer: Lateinit<ManagedBufferInstanceHandle<InstanceBufferType>>,

    pub projection: Lateinit<Projection>,
    pub camera_controller: Lateinit<CameraController>,

    pub camera_uniform: Lateinit<CameraUniform>,
    pub camera_buffer: Lateinit<ManagedBufferInstanceHandle<UniformBufferType>>,

    pub point_light_buffer: Lateinit<ManagedBufferInstanceHandle<StorageBufferType>>,
    pub spot_light_buffer: Lateinit<ManagedBufferInstanceHandle<StorageBufferType>>,

    pub point_light_uniform: Lateinit<Vec<PointLightUniform>>,
    pub spot_light_uniform: Lateinit<Vec<SpotLightUniform>>,

    pub point_light_count_buffer: Lateinit<ManagedBufferInstanceHandle<UniformBufferType>>,
    pub spot_light_count_buffer: Lateinit<ManagedBufferInstanceHandle<UniformBufferType>>,

    pub point_light_count_uniform: Lateinit<[u32; 4]>,
    pub spot_light_count_uniform: Lateinit<[u32; 4]>,

    pub clear_color: Lateinit<Color>,

    pub draw_passes: Vec<&'static mut DrawPass>,

    pub text_state: Lateinit<TextState>,
}
impl Clone for RenderManagerResource {
    fn clone(&self) -> Self {
        Self::default()
    }
}
