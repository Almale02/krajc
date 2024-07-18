use std::ops::Deref;

use image::GenericImageView;

use rapier3d::na::Point3;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    *,
};
use winit::window::Window;

use crate::{
    engine_runtime::EngineRuntime,
    rendering::{
        aspect_ratio::AspectUniform,
        buffer_manager::managed_buffer::{ManagedBufferGeneric, ManagedBufferInstanceHandle},
        camera::camera::{CameraController, CameraUniform, Projection, RenderCamera},
        material::TextureMaterial,
        mesh::mesh::TextureVertex,
        render_entity::{
            instancing::TestInstanceSchemes,
            render_entity::{RawTextureMaterialInstance, TextureMaterialInstance},
        },
        texture::texture::Texture,
    },
    InstanceBufferType, UniformBufferType, ENGINE_RUNTIME,
};

use super::RenderManagerResource;

impl EngineRuntime {
    pub async fn init_rendering(window: Window) {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = Instance::new(InstanceDescriptor {
            backends: Backends::all(),
            ..Default::default()
        });

        let surface = unsafe { instance.create_surface(&window) }.expect("failed");

        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("failed");

        drop(instance);
        let limits = Limits::downlevel_webgl2_defaults();
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: Features::empty(),
                    limits,
                    label: None,
                },
                None,
            )
            .await
            .expect("failed");

        let camera_uniform = CameraUniform::default();

        let _camera_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });
        let render_state = unsafe { ENGINE_RUNTIME.get().get_resource::<RenderManagerResource>() };

        render_state.device.init(device);
        let device = &render_state.device;

        let instance_scheme = TestInstanceSchemes::row(1);
        let instance_buffer = ManagedBufferInstanceHandle::<InstanceBufferType>::new_with_size(
            "instance_buffer".to_owned(),
            //4092u64.pow(2),
            268435456,
        );

        let camera_buffer = ManagedBufferInstanceHandle::<UniformBufferType>::new_with_init(
            "camera_buffer".to_owned(),
            camera_uniform,
        );
        let camera_buffer_actual = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("camera_buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: UniformBufferType::buffer_usages(),
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                //resource: camera_buffer.get_buffer().as_entire_binding(),
                resource: camera_buffer.get_buffer().as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        let aspect_uniform = AspectUniform::new();

        let aspect_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

        let aspect_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Aspect Buffer"),
            contents: bytemuck::cast_slice(&[aspect_uniform]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let aspect_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &aspect_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: aspect_buffer.as_entire_binding(),
            }],
            label: Some("aspect_bind_group"),
        });

        let mut texture =
            Texture::from_path("resources/image/dirt/dirt.png", &device, &queue).expect("failed");
        texture.texture_bind_group = Some(texture.get_texture_bind_group(&device));

        let surface_caps = surface.get_capabilities(&adapter);
        // Shader code in this tutorial assumes an sRGB surface texture. Using a different
        // one will result in all the colors coming out darker. If you want to support non
        // sRGB surfaces, you'll need to account for that when drawing to the frame.
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };

        surface.configure(&device, &config);

        let file =
            std::fs::read_to_string("resources/shaders/shader_texture.wgsl").expect("failed");
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Shader"),
            source: ShaderSource::Wgsl(file.as_str().into()),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    &Texture::get_texture_bind_layout(&device),
                    &camera_bind_group_layout,
                    &aspect_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
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
                    format: config.format,
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

        let camera = RenderCamera::new(
            //(0.0f32, 5.0f32, 10.0f32).into(),
            Point3::<f32>::new(0., 5., 10.),
            cgmath::Deg(-90.0).into(),
            cgmath::Deg(-20.0).into(),
        );
        let projection =
            Projection::new(config.width, config.height, cgmath::Deg(45.0), 0.1, 100.0);
        let camera_controller = CameraController::new(4.0, 0.4);

        let depth_texture = Texture::create_depth_texture(&device, &config, "Depth Texture");

        render_state.window.init(window);
        render_state.surface.init(surface);
        render_state.queue.init(queue);
        render_state.config.init(config);
        render_state.size.init(size);
        render_state.texture.init(texture);
        render_state.depth_texture.init(depth_texture);
        render_state.instance_scheme.init(instance_scheme);
        render_state.instance_buffer.init(instance_buffer);
        render_state.camera.init(camera);
        render_state.projection.init(projection);
        render_state.camera_controller.init(camera_controller);
        render_state.camera_uniform.init(camera_uniform);
        render_state.camera_buffer.init(camera_buffer);
        render_state.camera_buffer_actual.init(camera_buffer_actual);
        render_state.camera_bind_group.init(camera_bind_group);
        render_state.aspect_uniform.init(aspect_uniform);
        render_state.aspect_buffer.init(aspect_buffer);
        render_state.aspect_bind_group.init(aspect_bind_group);
        render_state.clear_color.init(Color::BLACK);
        render_state.render_pipeline.init(render_pipeline);

        render_state.material.init(TextureMaterial::default());
        render_state
            .material
            .set_instance(render_state.instance_buffer.deref().clone());
    }
}
