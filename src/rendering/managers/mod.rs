use bevy_ecs::component::Component;
use krajc::EngineResource;
use wgpu::*;
use winit::{dpi::PhysicalSize, window::Window};

use crate::{
    engine_runtime::{
        schedule_manager::system_params::system_resource::EngineResource, EngineRuntime,
    },
    generate_state_struct, InstanceBufferType, Lateinit, UniformBufferType,
};

use super::{
    aspect_ratio::AspectUniform,
    buffer_manager::{dupe, managed_buffer::ManagedBufferInstanceHandle},
    camera::camera::{CameraController, CameraUniform, Projection},
    material::TextureMaterial,
    render_entity::render_entity::TextureMaterialInstance,
    texture::texture::Texture,
};

pub mod input;
pub mod new;
pub mod render;
pub mod resize;
pub mod update;
pub mod window;

type S = &'static str;

#[derive(Default)]
pub struct RenderManagerResource<'w> {
    pub adapter: Lateinit<Adapter>,
    pub surface: Lateinit<Surface>,
    pub device: Lateinit<Device>,
    pub queue: Lateinit<Queue>,
    pub config: Lateinit<SurfaceConfiguration>,
    pub size: Lateinit<PhysicalSize<u32>>,
    pub render_pipeline: Lateinit<RenderPipeline>,

    pub texture: Lateinit<Texture>,
    pub depth_texture: Lateinit<Texture>,

    pub window: Lateinit<Window>,

    pub instance_scheme: Lateinit<Vec<TextureMaterialInstance>>,
    pub instance_buffer: Lateinit<ManagedBufferInstanceHandle<'w, InstanceBufferType>>,

    pub projection: Lateinit<Projection>,
    pub camera_controller: Lateinit<CameraController>,
    pub camera_uniform: Lateinit<CameraUniform>,
    pub camera_buffer: Lateinit<ManagedBufferInstanceHandle<'w, UniformBufferType>>,
    pub camera_buffer_actual: Lateinit<Buffer>,
    pub camera_bind_group: Lateinit<BindGroup>,

    pub aspect_uniform: Lateinit<AspectUniform>,
    pub aspect_buffer: Lateinit<Buffer>,
    pub aspect_bind_group: Lateinit<BindGroup>,

    pub clear_color: Lateinit<Color>,

    pub material: Lateinit<TextureMaterial<'w>>,
}
impl<'w> Clone for RenderManagerResource<'w> {
    fn clone(&self) -> Self {
        Self::default()
    }
}

impl<'a: 'static> EngineResource for RenderManagerResource<'a> {
    fn get_mut<'w>(engine: &'w mut EngineRuntime<'w>) -> &'w mut Self {
        let op = dupe(engine)
            .static_resource_map
            .get_mut(&std::any::TypeId::of::<Self>());
        match op {
            Some(val) => unsafe { val.downcast_mut_unchecked() },
            None => {
                dupe(engine).static_resource_map.insert(
                    std::any::TypeId::of::<Self>(),
                    Box::new(RenderManagerResource::default()),
                );

                unsafe {
                    dupe(engine)
                        .static_resource_map
                        .get_mut(&std::any::TypeId::of::<Self>())
                        .unwrap()
                        .downcast_mut_unchecked()
                }
            }
        }
    }
    fn get<'w>(engine: &'w mut crate::engine_runtime::EngineRuntime<'w>) -> &'w Self {
        let op = crate::rendering::buffer_manager::dupe(engine)
            .static_resource_map
            .get_mut(&std::any::TypeId::of::<Self>());
        match op {
            Some(val) => unsafe { val.downcast_mut_unchecked() },
            None => {
                //let addr = crate::TypedAddr::new_with_ref(new).addr;
                crate::rendering::buffer_manager::dupe(engine)
                    .static_resource_map
                    .insert(
                        std::any::TypeId::of::<Self>(),
                        Box::new(RenderManagerResource::default()),
                    );

                unsafe {
                    crate::rendering::buffer_manager::dupe(engine)
                        .static_resource_map
                        .get_mut(&std::any::TypeId::of::<Self>())
                        .unwrap()
                        .downcast_mut_unchecked()
                }
            }
        }
    }
}
