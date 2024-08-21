use std::{collections::HashMap, ops::Range};

use bevy_ecs::{entity::Entity, query::With};
use krajc::{system_fn, EngineResource, FromEngine};
use uuid::Uuid;
use wgpu::{
    BindGroupLayout, Buffer, CompareFunction, DepthBiasState, DepthStencilState, Face, RenderPass,
    RenderPipeline, RenderPipelineDescriptor, ShaderModule, StencilState,
};

use crate::{
    drop_span,
    engine_runtime::schedule_manager::system_params::system_local::Local,
    rendering::{
        asset::{AssetHandle, AssetHandleUntype},
        draw_pass::DrawPass,
    },
    span, ENGINE_RUNTIME,
};
#[allow(unused_imports)]
use crate::{
    engine_runtime::{
        schedule_manager::system_params::{
            system_query::SystemQuery,
            system_resource::{EngineResource, Res},
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
    typed_addr::dupe,
    FromEngine, Lateinit, LightMaterialMarker,
};

use super::instance_data::{LightMaterialInstance, RawLightMaterialInstance};

#[derive(FromEngine)]
pub struct LightMaterial {
    pub instance_count: u32,
    mesh: AssetHandle<Mesh<TextureVertex>>,
    texture: AssetHandle<Texture>,
    instance_buffer: Lateinit<ManagedBufferInstanceHandle<InstanceBufferType>>,
    camera_layout: Lateinit<&'static wgpu::BindGroupLayout>,
    camera_bind_group: Lateinit<wgpu::BindGroup>,
    point_light_layout: Lateinit<&'static wgpu::BindGroupLayout>,
    point_light_bind_group: Lateinit<wgpu::BindGroup>,
    spot_light_layout: Lateinit<&'static wgpu::BindGroupLayout>,
    spot_light_bind_group: Lateinit<wgpu::BindGroup>,
}

#[derive(EngineResource)]
pub struct LightMaterialResource {
    pub point_light_layout: BindGroupLayout,
    pub spot_light_layout: BindGroupLayout,
    pub camera_layout: BindGroupLayout,
    pub pipeline: Lateinit<RenderPipeline>,
    pub shader_asset_handle: AssetHandleUntype,
}

impl FromEngine for LightMaterialResource {
    fn from_engine(engine: &'static mut EngineRuntime) -> Self {
        let render = engine.get_resource::<RenderManagerResource>();
        Self {
            shader_asset_handle: AssetHandleUntype::from_engine(engine),
            camera_layout: {
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
                    })
            },
            point_light_layout: {
                render
                    .device
                    .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                        entries: &[
                            wgpu::BindGroupLayoutEntry {
                                binding: 0,
                                visibility: wgpu::ShaderStages::FRAGMENT,
                                ty: wgpu::BindingType::Buffer {
                                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                                    has_dynamic_offset: false,
                                    min_binding_size: None,
                                },
                                count: None,
                            },
                            wgpu::BindGroupLayoutEntry {
                                binding: 1,
                                visibility: wgpu::ShaderStages::FRAGMENT,
                                ty: wgpu::BindingType::Buffer {
                                    ty: wgpu::BufferBindingType::Uniform,
                                    has_dynamic_offset: false,
                                    min_binding_size: None,
                                },
                                count: None,
                            },
                        ],
                        label: Some("point_light_bind_group_layout"),
                    })
            },
            spot_light_layout: {
                render
                    .device
                    .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                        entries: &[
                            wgpu::BindGroupLayoutEntry {
                                binding: 0,
                                visibility: wgpu::ShaderStages::FRAGMENT,
                                ty: wgpu::BindingType::Buffer {
                                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                                    has_dynamic_offset: false,
                                    min_binding_size: None,
                                },
                                count: None,
                            },
                            wgpu::BindGroupLayoutEntry {
                                binding: 1,
                                visibility: wgpu::ShaderStages::FRAGMENT,
                                ty: wgpu::BindingType::Buffer {
                                    ty: wgpu::BufferBindingType::Uniform,
                                    has_dynamic_offset: false,
                                    min_binding_size: None,
                                },
                                count: None,
                            },
                        ],
                        label: Some("spot_light_bind_group_layout"),
                    })
            },
            pipeline: Default::default(),
        }
    }
}

impl LightMaterial {
    pub fn set_mesh(&mut self, mesh: AssetHandle<Mesh<TextureVertex>>) {
        self.mesh = mesh;
    }
    pub fn set_texture(&mut self, texture: AssetHandle<Texture>) {
        self.texture = texture;
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
        let iter_part = data.iter();

        let map_part = iter_part.map(|arg| arg.to_raw());

        let collect_part = map_part.collect::<Vec<_>>();

        self.instance_buffer.set_data_vec(collect_part);
    }
    pub fn set_render_pipeline(
        engine: &'static mut EngineRuntime,
        shader: &ShaderModule,
        shader_handle: AssetHandleUntype,
    ) {
        let render = engine.get_resource_mut::<RenderManagerResource>();
        let material_res = engine.get_resource_mut::<LightMaterialResource>();

        material_res.shader_asset_handle = shader_handle;

        let render_pipeline_layout =
            render
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Light Render Pipeline Layout"),
                    bind_group_layouts: &[
                        &Texture::get_texture_bind_layout(&render.device),
                        &material_res.camera_layout,
                        &material_res.point_light_layout,
                        &material_res.spot_light_layout,
                    ],
                    push_constant_ranges: &[],
                });

        material_res
            .pipeline
            .set(
                render
                    .device
                    .create_render_pipeline(&RenderPipelineDescriptor {
                        label: Some("Light Render Pipeline"),
                        layout: Some(&render_pipeline_layout),
                        vertex: wgpu::VertexState {
                            module: shader,
                            entry_point: "vs_main", // 1.
                            buffers: &[TextureVertex::layout(), RawLightMaterialInstance::desc()], // 2.
                        },
                        fragment: Some(wgpu::FragmentState {
                            // 3.
                            module: shader,
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
                    }),
            );
    }
}

#[system_fn(RuntimePostPhysicsSyncSchedule)]
pub fn update_light_material(
    mut query: SystemQuery<
        (
            Entity,
            &Transform,
            &AssetHandle<Texture>,
            &AssetHandle<Mesh<TextureVertex>>,
        ),
        With<LightMaterialMarker>,
    >,
    mut render: Res<RenderManagerResource>,
    mut local_passes: Local<Vec<DrawPass>>,
    mut instance_buffers: Local<Vec<ManagedBufferInstanceHandle<InstanceBufferType>>>,
) {
    local_passes.clear();
    let mut instance_datas: HashMap<
        (&AssetHandle<Texture>, &AssetHandle<Mesh<TextureVertex>>),
        Vec<LightMaterialInstance>,
    > = HashMap::new();
    span!(trace_createing_instances, "creating instances");
    query
        .iter()
        .for_each(|(_entity, trans, texture_handle, mesh_handle)| {
            span!(trace_inside, "inside foreach loop");
            let hash = (texture_handle, mesh_handle);

            match instance_datas.get_mut(&hash) {
                Some(x) => x.push(LightMaterialInstance::new(trans.clone())),
                None => {
                    instance_datas.insert(hash, vec![LightMaterialInstance::new(trans.clone())]);
                }
            }
        });
    drop_span!(trace_createing_instances);

    instance_datas
        .iter()
        .enumerate()
        .for_each(|(idx, ((texture, mesh), value))| {
            if instance_buffers.len() <= idx {
                instance_buffers.push(ManagedBufferInstanceHandle::new_with_size(
                    Uuid::new_v4().to_string(),
                    1024_u64.pow(2),
                ));
            }

            let mut material = LightMaterial::from_engine(unsafe { ENGINE_RUNTIME.get() });
            material.set_texture((*texture).clone());
            material.set_mesh((*mesh).clone());
            material.set_instance(instance_buffers[idx].clone());
            material.set_instance_value_ref(value.iter().collect());

            let draw_pass = DrawPass::new(
                Box::new(material),
                vec![texture.as_untype(), mesh.as_untype()],
            );
            local_passes.push(draw_pass);
            render
                .draw_passes
                .push(local_passes.get_mut().last_mut().unwrap());
        });
}

impl MaterialGeneric for LightMaterial {
    fn get_instance_range(&self) -> Range<u32> {
        0..self.instance_count
    }
    fn render_pipeline(&mut self, engine: &mut EngineRuntime) -> &RenderPipeline {
        let light_res = engine.get_resource_mut::<LightMaterialResource>();
        light_res
            .pipeline
            .as_option()
            .expect("mateiral was rendered before the shader was loader")
    }
    fn index_buffer(&self, _engine: &mut EngineRuntime) -> &'static Buffer {
        &dupe(self).mesh.get().unwrap().index_buffer
    }
    fn vertex_buffer(&self, _engine: &mut EngineRuntime) -> &'static Buffer {
        &dupe(self).mesh.get().unwrap().vertex_buffer
    }
    fn instance_buffer(&self, _engine: &mut EngineRuntime) -> &'static Buffer {
        dupe(self).instance_buffer.get_buffer()
    }
    fn setup_bind_groups(&mut self, engine: &mut EngineRuntime) {
        let render = engine.get_resource_mut::<RenderManagerResource>();
        let light_res = engine.get_resource::<LightMaterialResource>();

        self.camera_layout.set(&light_res.camera_layout);
        self.point_light_layout.set(&light_res.point_light_layout);
        self.spot_light_layout.set(&light_res.spot_light_layout);

        self.camera_bind_group
            .set(render.device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &self.camera_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: render.camera_buffer.get_buffer().as_entire_binding(),
                }],
                label: Some("camera_bind_group"),
            }));
        self.point_light_bind_group.set(
            render.device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &self.point_light_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: render.point_light_buffer.get_buffer().as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: render
                            .point_light_count_buffer
                            .get_buffer()
                            .as_entire_binding(),
                    },
                ],
                label: Some("point_light_bind_group"),
            }),
        );
        self.spot_light_bind_group.set(
            render.device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &self.spot_light_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: render.spot_light_buffer.get_buffer().as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: render
                            .spot_light_count_buffer
                            .get_buffer()
                            .as_entire_binding(),
                    },
                ],
                label: Some("spot_light_bind_group"),
            }),
        );
    }
    fn set_bind_groups<'a>(&'a self, pipeline: &mut RenderPass<'a>, _engine: &mut EngineRuntime) {
        //let state = engine.get_resource_mut::<RenderManagerResource>();

        let texture = self
            .texture
            .get()
            .expect("could not access texture in set_bind_groups");

        pipeline.set_bind_group(0, texture.texture_bind_group.as_option().expect("the field texture_bind_group needs to be set on textures before applying them for render,
        you could call Texture::get_texture_bind_group on the created texture, and then set the return as the bind group field"), &[]);

        pipeline.set_bind_group(1, &self.camera_bind_group, &[]);
        pipeline.set_bind_group(2, &self.point_light_bind_group, &[]);
        pipeline.set_bind_group(3, &self.spot_light_bind_group, &[]);
    }
    fn get_index_range(&self) -> Range<u32> {
        0..self.mesh.get().unwrap().index_list.len() as u32
    }
    fn register_systems(&self, engine: &mut EngineRuntime) {
        //update_light_material!(engine);
    }

    fn get_shader_asset_handle(
        &self,
        engine: &mut EngineRuntime,
    ) -> crate::rendering::asset::AssetHandleUntype {
        let a = engine.get_resource::<LightMaterialResource>();

        a.shader_asset_handle.clone()
    }
}
