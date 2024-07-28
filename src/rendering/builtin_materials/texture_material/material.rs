use std::{ops::Range, time::Instant};

use bevy_ecs::entity::Entity;
use cgmath::Zero;
use krajc::system_fn;
use wgpu::{
    Buffer, CompareFunction, DepthBiasState, DepthStencilState, Face, RenderPass, RenderPipeline,
    RenderPipelineDescriptor, ShaderModuleDescriptor, ShaderSource, StencilState,
};

use crate::{
    engine_runtime::{
        schedule_manager::{
            runtime_schedule::RuntimeEndFrameSchedule,
            system_params::{system_query::SystemQuery, system_resource::Res},
        },
        EngineRuntime,
    },
    rendering::{
        buffer_manager::{managed_buffer::ManagedBufferInstanceHandle, InstanceBufferType},
        managers::RenderManagerResource,
        material::MaterialGeneric,
        mesh::mesh::{Mesh, TextureVertex},
        systems::general::Transform,
        texture::texture::Texture,
    },
    typed_addr::dupe,
    Lateinit,
};

use super::instance_data::{RawTextureMaterialInstance, TextureMaterialInstance};

#[derive(Default)]
pub struct TextureMaterial {
    pub instance_count: u32,
    mesh: Lateinit<Mesh>,
    instance_buffer: Lateinit<ManagedBufferInstanceHandle<InstanceBufferType>>,

    camera_layout: Lateinit<wgpu::BindGroupLayout>,
    camera_bind_group: Lateinit<wgpu::BindGroup>,
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
    pub fn set_instance_value(&mut self, data: Vec<TextureMaterialInstance>) {
        self.instance_count = data.len() as u32;

        //let start = Instant::now();

        let mut new = Vec::<RawTextureMaterialInstance>::new();

        for i in data {
            new.push(i.to_raw());
        }

        //dbg!(start.elapsed());
        self.instance_buffer.set_data_vec(new);
    }
    pub fn set_instance_value_ref(&mut self, data: Vec<&TextureMaterialInstance>) {
        self.instance_count = data.len() as u32;
        let iter_start = Instant::now();

        let iter_part = data.iter();

        println!("iter_start took {:?}", iter_start.elapsed());

        let map_start = Instant::now();

        let map_part = iter_part.map(|arg| arg.to_raw());

        println!("map took {:?}", map_start.elapsed());

        let collect_start = Instant::now();

        let collect_part = map_part.collect::<Vec<_>>();

        println!("collect took {:?}", collect_start.elapsed());

        println!("everything took {:?}", iter_start.elapsed());

        self.instance_buffer.set_data_vec(collect_part);
    }
}

#[system_fn(RuntimeEndFrameSchedule)]
pub fn update_texture_material(
    mut query: SystemQuery<
        (Entity, &Transform),
        //(Or<(Changed<Transform>, Added<Transform>)>, With<Marker>),
        //(Or<(Added<Transform>, Changed<Transform>)>, With<Marker>),
    >,
    mut render: Res<RenderManagerResource>,
) {
    let query = query
        .iter()
        .map(|(entity, trans)| {
            //println!("{:?} has trans: {}", entity, trans.iso);
            TextureMaterialInstance::new(
                trans.clone().into(),
                cgmath::Quaternion::new(
                    trans.rotation.w,
                    trans.rotation.i,
                    trans.rotation.j,
                    trans.rotation.k,
                ),
            )
        })
        .collect::<Vec<_>>();
    if !query.len().is_zero() {
        render.material.set_instance_value(query)
    };
}

impl MaterialGeneric for TextureMaterial {
    fn get_instance_range(&self) -> Range<u32> {
        0..self.instance_count
    }
    fn render_pipeline(&self, engine: &mut EngineRuntime) -> RenderPipeline {
        let render = engine.get_resource_mut::<RenderManagerResource>();

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
        let render_pipeline_layout =
            render
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &[
                        &Texture::get_texture_bind_layout(&render.device),
                        &camera_bind_group_layout,
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
                    polygon_mode: wgpu::PolygonMode::Line,
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
    fn setup_bind_groups(&mut self, engine: &mut EngineRuntime) { //
    }
    fn index_buffer(&self, _engine: &mut EngineRuntime) -> &'static Buffer {
        &dupe(self).mesh.index_buffer
    }
    fn vertex_buffer(&self, _engine: &mut EngineRuntime) -> &'static Buffer {
        &dupe(self).mesh.vertex_buffer
    }
    fn instance_buffer(&self, _engine: &mut EngineRuntime) -> &'static Buffer {
        dupe(self).instance_buffer.get_buffer()
    }
    fn set_bind_groups<'a>(&'a self, pipeline: &mut RenderPass<'a>, engine: &mut EngineRuntime) {
        let state = engine.get_resource_mut::<RenderManagerResource>();

        pipeline.set_bind_group(0, state.texture.texture_bind_group.as_ref().expect("the field texture_bind_group needs to be set on textures before applying them for render,
you could call Texture::get_texture_bind_group on the created texture, and then set the return as the bind group field"), &[]);
        pipeline.set_bind_group(1, &self.camera_bind_group, &[]);
    }
    fn get_index_range(&self) -> Range<u32> {
        0..self.mesh.index_list.len() as u32
    }
    fn register_systems(&self, engine: &mut EngineRuntime) {
        update_texture_material!(engine);
    }
}
