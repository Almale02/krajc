use std::{
    ops::{DerefMut, Range},
    time::Instant,
};

use bevy_ecs::{
    entity::Entity,
    query::{Added, Changed, Or, With, Without},
};
use cgmath::Zero;
use krajc::{system_fn, FromEngine};
use wgpu::{
    Buffer, CompareFunction, DepthBiasState, DepthStencilState, Face, RenderPass, RenderPipeline,
    RenderPipelineDescriptor, ShaderModuleDescriptor, ShaderSource, StencilState,
};

use crate::{
    engine_runtime::{
        engine_cache::engine_cache::{CacheHandle, EngineCache},
        schedule_manager::{
            runtime_schedule::{RuntimePhysicsSyncMainSchedule, RuntimePostPhysicsSyncSchedule},
            system_params::{system_query::SystemQuery, system_resource::Res},
        },
        EngineRuntime,
    },
    rendering::{
        buffer_manager::{managed_buffer::ManagedBufferInstanceHandle, InstanceBufferType},
        builtin_materials::light_material::instance_data::LightMaterialInstance,
        managers::RenderManagerResource,
        material::MaterialGeneric,
        mesh::mesh::{Mesh, TextureVertex, Vertex},
        systems::general::Transform,
        texture::texture::Texture,
    },
    typed_addr::dupe,
    Lateinit, TextureMaterialMarker,
};

use super::instance_data::{RawTextureMaterialInstance, TextureMaterialInstance};

#[derive(FromEngine)]
pub struct TextureMaterial {
    pub instance_count: u32,
    mesh: Lateinit<Mesh<TextureVertex>>,
    instance_buffer: Lateinit<ManagedBufferInstanceHandle<InstanceBufferType>>,

    camera_layout: Lateinit<wgpu::BindGroupLayout>,
    camera_bind_group: Lateinit<wgpu::BindGroup>,

    pipeline: CacheHandle<RenderPipeline>,
}
impl TextureMaterial {
    pub fn set_mesh(&mut self, mesh: Mesh<TextureVertex>) {
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
        self.instance_buffer
            .set_data_vec(data.iter().map(TextureMaterialInstance::to_raw).collect());
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

#[system_fn(RuntimePostPhysicsSyncSchedule)]
pub fn update_texture_material(
    mut query: SystemQuery<(Entity, &Transform), (With<TextureMaterialMarker>, Changed<Transform>)>,
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
        render.texture_material.set_instance_value(query)
    };
}

impl MaterialGeneric for TextureMaterial {
    fn get_instance_range(&self) -> Range<u32> {
        0..self.instance_count
    }
    fn render_pipeline(&mut self, engine: &mut EngineRuntime) -> &RenderPipeline {
        self.pipeline.cache(|| {
            let render = engine.get_resource_mut::<RenderManagerResource>();

            let file =
                std::fs::read_to_string("resources/shaders/shader_texture.wgsl").expect("failed");
            let shader = render.device.create_shader_module(ShaderModuleDescriptor {
                label: Some("Shader Texture"),
                source: ShaderSource::Wgsl(file.as_str().into()),
            });

            let render_pipeline_layout =
                render
                    .device
                    .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                        label: Some("Texture Render Pipeline Layout"),
                        bind_group_layouts: &[
                            &Texture::get_texture_bind_layout(&render.device),
                            &self.camera_layout,
                        ],
                        push_constant_ranges: &[],
                    });

            render
                .device
                .create_render_pipeline(&RenderPipelineDescriptor {
                    label: Some("Texture Render Pipeline"),
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
                })
        })
    }
    fn setup_bind_groups(&mut self, engine: &mut EngineRuntime) {
        let render = engine.get_resource_mut::<RenderManagerResource>();

        self.camera_layout
            .set(
                render
                    .device
                    .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                        entries: &[wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        }],
                        label: Some("camera_bind_group_layout"),
                    }),
            );
        self.camera_bind_group
            .set(render.device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &self.camera_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    //resource: camera_buffer.get_buffer().as_entire_binding(),
                    resource: render.camera_buffer.get_buffer().as_entire_binding(),
                }],
                label: Some("camera_bind_group"),
            }));
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

        pipeline.set_bind_group(0, state.texture.texture_bind_group.as_option().expect("the field texture_bind_group needs to be set on textures before applying them for render,
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
