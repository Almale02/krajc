use std::ops::{Deref, Range};

use cgmath::Zero;
use krajc::{system_fn, system_fn_non_expand, Comp};
use legion::{query::EntityFilterTuple, Query, Read};
use wgpu::*;

use crate::{
    engine_runtime::{
        schedule_manager::{
            runtime_schedule::RuntimeUpdateSchedule,
            system_params::{
                system_query::{EcsWorld, SystemQuery},
                system_resource::Res,
            },
        },
        EngineRuntime,
    },
    InstanceBufferType, Lateinit,
};

use super::{
    buffer_manager::managed_buffer::ManagedBufferInstanceHandle,
    managers::RenderManagerResource,
    mesh::mesh::{Mesh, TextureVertex},
    render_entity::render_entity::{RawTextureMaterialInstance, TextureMaterialInstance},
    texture::texture::Texture,
};

pub trait MaterialGeneric {
    fn render_pipeline(&self, engine: &mut EngineRuntime) -> RenderPipeline;
    fn vertex_buffer(&'static self, engine: &mut EngineRuntime) -> &'static Buffer;
    fn index_buffer(&'static self, engine: &mut EngineRuntime) -> &'static Buffer;
    fn instance_buffer(&'static self, engine: &mut EngineRuntime) -> &'static Buffer;
    fn set_bind_groups(&self, pipeline: &mut RenderPass<'_>, engine: &mut EngineRuntime);
    fn get_index_range(&self) -> Range<u32>;
    fn register_systems(&self, engine: &mut EngineRuntime);
}

#[derive(Default)]
pub struct TextureMaterial {
    mesh: Lateinit<Mesh>,
    instance_buffer: Lateinit<ManagedBufferInstanceHandle<InstanceBufferType>>,
}
impl TextureMaterial {
    pub fn set_mesh(&mut self, mesh: Mesh) {
        self.mesh.set(mesh);
    }
    pub fn set_instance(
        &mut self,
        instance_buffer: ManagedBufferInstanceHandle<InstanceBufferType>,
    ) {
        self.instance_buffer.set(instance_buffer);
    }
}

#[system_fn(RuntimeUpdateSchedule)]
pub fn update_texture_material(
    query: SystemQuery<Read<TextureMaterialInstance>>,
    mut render: Res<RenderManagerResource>,
    world: EcsWorld,
) {
    let render = render.get_static_mut();
    let mut iter: Vec<&TextureMaterialInstance> = vec![];

    unsafe {
        for chunk in query.query().iter_chunks_unchecked(&*world) {
            let chunk = chunk.get_indexable();

            for entity in chunk {
                iter.push(entity);
            }
        }
    }

    if iter.len().is_zero() {
        dbg!("zero");
    }

    let instances_raw = iter
        .into_iter()
        .map(TextureMaterialInstance::to_raw)
        .collect::<Vec<_>>();

    render.instance_buffer.set_data_vec(instances_raw);
}
impl MaterialGeneric for TextureMaterial {
    fn render_pipeline(&self, engine: &mut EngineRuntime) -> RenderPipeline {
        let render = engine.get_resource::<RenderManagerResource>();

        std::fs::read_to_string("resources/shaders/shader_texture.wgsl").expect("failed");
        let file =
            std::fs::read_to_string("resources/shaders/shader_texture.wgsl").expect("failed");
        let shader = render.device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Shader"),
            source: ShaderSource::Wgsl(file.as_str().into()),
        });

        let camera_bind_group_layout =
            render
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                    label: Some("camera_bind_group_layout"),
                });
        let aspect_bind_group_layout =
            render
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                    label: Some("aspect_bind_group_layout"),
                });
        let render_pipeline_layout =
            render
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &[
                        &Texture::get_texture_bind_layout(&render.device),
                        &camera_bind_group_layout,
                        &aspect_bind_group_layout,
                    ],
                    push_constant_ranges: &[],
                });

        let render_pipeline = render
            .device
            .create_render_pipeline(&RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main", // 1.
                    buffers: &[TextureVertex::layout(), RawTextureMaterialInstance::desc()], // 2.
                },
                fragment: Some(wgpu::FragmentState {
                    // 3.
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        // 4.
                        format: render.config.format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList, // 1.
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Cw, // 2.
                    cull_mode: Some(Face::Front),
                    // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                    polygon_mode: wgpu::PolygonMode::Fill,
                    // Requires Features::DEPTH_CLIP_CONTROL
                    unclipped_depth: false,
                    // Requires Features::CONSERVATIVE_RASTERIZATION
                    conservative: false,
                },
                depth_stencil: Some(DepthStencilState {
                    format: Texture::DEPTH_FORMAT,
                    depth_write_enabled: true,
                    depth_compare: CompareFunction::Less,
                    stencil: StencilState::default(),
                    bias: DepthBiasState::default(),
                }), // 1.
                multisample: wgpu::MultisampleState {
                    count: 1,                         // 2.
                    mask: !0,                         // 3.
                    alpha_to_coverage_enabled: false, // 4.
                },
                multiview: None, // 5.
            });
        render_pipeline
    }
    fn index_buffer(&'static self, _engine: &mut EngineRuntime) -> &'static Buffer {
        &self.mesh.index_buffer
    }
    fn vertex_buffer(&'static self, _engine: &mut EngineRuntime) -> &'static Buffer {
        &self.mesh.vertex_buffer
    }
    fn instance_buffer(&'static self, _engine: &mut EngineRuntime) -> &'static Buffer {
        self.instance_buffer.get_buffer()
    }
    fn set_bind_groups(&self, pipeline: &mut RenderPass<'_>, engine: &mut EngineRuntime) {
        let state = engine.get_resource::<RenderManagerResource>();

        pipeline.set_bind_group(0, state.texture.texture_bind_group.as_ref().expect("the field texture_bind_group needs to be set on textures before applying them for render,
you could call Texture::get_texture_bind_group on the created texture, and then set the return as the bind group field"), &[]);
        pipeline.set_bind_group(1, &state.camera_bind_group, &[]);
        pipeline.set_bind_group(2, &state.aspect_bind_group, &[]);
    }
    fn get_index_range(&self) -> Range<u32> {
        0..self.mesh.index_list.len() as u32
    }
    fn register_systems(&self, engine: &mut EngineRuntime) {
        update_texture_material!(engine);
    }
}
