use wgpu::*;

use crate::{
    engine_runtime::{
        engine_state_manager::generic_state_manager::GenericStateRefTemplate,
        schedule_manager::system_params::system_resource::EngineResource, EngineRuntime,
    },
    generate_state_struct, InstanceBufferType, UniformBufferType,
};

use super::{
    aspect_ratio::AspectUniform,
    buffer_manager::managed_buffer::ManagedBufferInstanceHandle,
    camera::camera::{Camera, CameraController, CameraUniform, Projection},
    material::TextureMaterial,
    render_entity::render_entity::RenderEntity,
    texture::texture::Texture,
};

pub mod input;
pub mod new;
pub mod render;
pub mod resize;
pub mod update;
pub mod window;

type S = &'static str;

generate_state_struct!(RenderManagerResource {
     adapter: Adapter = "adapter",
     surface: Surface = "surface",
     device: Device = "device",
     queue: Queue = "queue",
     config: SurfaceConfiguration = "config",
     size: winit::dpi::PhysicalSize<u32> = "size",
     render_pipeline: RenderPipeline = "render_pipeline",

     texture: Texture = "texture",
     depth_texture: Texture = "depth_texture",

     window: winit::window::Window = "window",

     instance_scheme: Vec<RenderEntity> = "instance_scheme",
     instance_buffer: ManagedBufferInstanceHandle<InstanceBufferType>= "instance_buffer",

     camera: Camera = "camera",
     projection: Projection = "projection",
     camera_controller: CameraController = "camera_controller",
     camera_uniform: CameraUniform = "camera_uniform",
     camera_buffer: ManagedBufferInstanceHandle<UniformBufferType> = "camera_buffer",
     camera_buffer_actual: Buffer = "camera_buffer_actual",
     camera_bind_group: BindGroup = "camera_bind_group",

     aspect_uniform: AspectUniform = "aspect_uniform",
     aspect_buffer: Buffer = "aspect_buffer",
     aspect_bind_group: BindGroup = "aspect_bind_group",

     clear_color: Color = "clear_color",

     material: TextureMaterial = "texture_material",

});
impl Clone for RenderManagerResource {
    fn clone(&self) -> Self {
        Self::default()
    }
}
