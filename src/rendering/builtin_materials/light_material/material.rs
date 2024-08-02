use std::{ops::Range, time::Instant};

use bevy_ecs::{
    entity::Entity,
    query::{Added, Changed, Or, With, Without},
};
use cgmath::Zero;
use krajc::{system_fn, FromEngine};
use wgpu::{
    Buffer, CompareFunction, DepthBiasState, DepthStencilState, Face, Features, RenderPass,
    RenderPipeline, RenderPipelineDescriptor, ShaderModuleDescriptor, ShaderSource, StencilState,
};

use crate::{
    drop_span,
    engine_runtime::{
        engine_cache::engine_cache::CacheHandle,
        schedule_manager::{
            runtime_schedule::{RuntimePhysicsSyncMainSchedule, RuntimePostPhysicsSyncSchedule},
            system_params::{system_query::SystemQuery, system_resource::Res},
        },
        EngineRuntime,
    },
    rendering::{
        buffer_manager::{managed_buffer::ManagedBufferInstanceHandle, InstanceBufferType},
        managers::RenderManagerResource,
        material::MaterialGeneric,
        mesh::mesh::{Mesh, TextureVertex, Vertex},
        systems::general::Transform,
        texture::texture::Texture,
    },
    span,
    typed_addr::dupe,
    Lateinit, LightMaterialMarker, TextureMaterialMarker,
};

use super::instance_data::{LightMaterialInstance, RawLightMaterialInstance};

#[derive(FromEngine)]
pub struct LightMaterial {
    pub instance_count: u32,
    mesh: Lateinit<Mesh<TextureVertex>>,
    instance_buffer: Lateinit<ManagedBufferInstanceHandle<InstanceBufferType>>,
    camera_layout: Lateinit<wgpu::BindGroupLayout>,
    camera_bind_group: Lateinit<wgpu::BindGroup>,
    light_layout: Lateinit<wgpu::BindGroupLayout>,
    light_bind_group: Lateinit<wgpu::BindGroup>,
    pipeline: CacheHandle<RenderPipeline>,
}
impl LightMaterial {
    pub fn set_mesh(&mut self, mesh: Mesh<TextureVertex>) {
        self.mesh.set(mesh);
    }
    pub fn set_instance(
        &mut self,
        instance_buffer: ManagedBufferInstanceHandle<InstanceBufferType>,
    ) {
        self.instance_buffer.set(instance_buffer);
    }
    pub fn set_instance_value(&mut self, data: Vec<LightMaterialInstance>) {
        self.instance_count = data.len() as u32;
        self.instance_buffer
            .set_data_vec(data.iter().map(LightMaterialInstance::to_raw).collect());
    }
    pub fn set_instance_value_ref(&mut self, data: Vec<&LightMaterialInstance>) {
        self.instance_count = data.len() as u32;
        let iter_start = Instant::now();

        let iter_part = data.iter();

        let map_part = iter_part.map(|arg| arg.to_raw());

        let collect_part = map_part.collect::<Vec<_>>();

        self.instance_buffer.set_data_vec(collect_part);
    }
}

#[system_fn(RuntimePostPhysicsSyncSchedule)]
pub fn update_light_material(
    mut query: SystemQuery<(Entity, &Transform), (With<LightMaterialMarker>, Changed<Transform>)>,
    mut render: Res<RenderManagerResource>,
) {
    let query = query
        .iter()
        .map(|(entity, trans)| {
            //println!("{:?} has trans: {}", entity, trans.iso);
            LightMaterialInstance::new(
                trans.clone().into(),
                //cgmath::Quaternion::zero(),
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
        render.light_material.set_instance_value(query)
    };
}

impl MaterialGeneric for LightMaterial {
    fn get_instance_range(&self) -> Range<u32> {
        0..self.instance_count
    }
    fn render_pipeline(&mut self, engine: &mut EngineRuntime) -> &RenderPipeline {
        self.pipeline.cache(|| {
            let render = engine.get_resource_mut::<RenderManagerResource>();

            span!(trace_reading_shader, "reading shader");
            let file = std::fs::read_to_string("resources/shaders/shader_light.wgsl")
                .expect("failed to load shader for light material");
            drop_span!(trace_reading_shader);
            let shader = render.device.create_shader_module(ShaderModuleDescriptor {
                label: Some("Shader"),
                source: ShaderSource::Wgsl(file.as_str().into()),
            });

            let render_pipeline_layout =
                render
                    .device
                    .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                        label: Some("Render Pipeline Layout"),
                        bind_group_layouts: &[
                            &Texture::get_texture_bind_layout(&render.device),
                            &self.camera_layout,
                            &self.light_layout,
                        ],
                        push_constant_ranges: &[],
                    });

            render
                .device
                .create_render_pipeline(&RenderPipelineDescriptor {
                    label: Some("Render Pipeline"),
                    layout: Some(&render_pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &shader,
                        entry_point: "vs_main", // 1.
                        buffers: &[TextureVertex::layout(), RawLightMaterialInstance::desc()], // 2.
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
    fn index_buffer(&self, _engine: &mut EngineRuntime) -> &'static Buffer {
        &dupe(self).mesh.index_buffer
    }
    fn vertex_buffer(&self, _engine: &mut EngineRuntime) -> &'static Buffer {
        &dupe(self).mesh.vertex_buffer
    }
    fn instance_buffer(&self, _engine: &mut EngineRuntime) -> &'static Buffer {
        dupe(self).instance_buffer.get_buffer()
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
        self.light_layout
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
                        label: Some("light_bind_group_layout"),
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
        self.light_bind_group
            .set(render.device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &self.light_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    //resource: camera_buffer.get_buffer().as_entire_binding(),
                    resource: render.light_buffer.get_buffer().as_entire_binding(),
                }],
                label: Some("light_bind_group"),
            }));
    }
    fn set_bind_groups<'a>(&'a self, pipeline: &mut RenderPass<'a>, engine: &mut EngineRuntime) {
        let state = engine.get_resource_mut::<RenderManagerResource>();

        pipeline.set_bind_group(0, state.texture.texture_bind_group.as_option().expect("the field texture_bind_group needs to be set on textures before applying them for render,
you could call Texture::get_texture_bind_group on the created texture, and then set the return as the bind group field"), &[]);
        pipeline.set_bind_group(1, &self.camera_bind_group, &[]);
        pipeline.set_bind_group(2, &self.light_bind_group, &[]);
    }
    fn get_index_range(&self) -> Range<u32> {
        0..self.mesh.index_list.len() as u32
    }
    fn register_systems(&self, engine: &mut EngineRuntime) {
        update_light_material!(engine);
    }
}

#[repr(C)]
#[derive(Default, Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightUniform {
    pub position: [f32; 4],
    pub color: [f32; 4],
    pub rot_unit: [f32; 4],
}

impl LightUniform {
    pub fn new(position: [f32; 3], color: [f32; 3], rot: [f32; 3]) -> Self {
        Self {
            position: [position[0], position[1], position[2], 0.],
            color: [color[0], color[1], color[2], 0.],
            rot_unit: [rot[0], rot[1], rot[2], 0.],
        }
    }
}
