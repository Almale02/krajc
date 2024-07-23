//use core::resource::Texture;

use wgpu::*;

use crate::{
    engine_runtime::EngineRuntime,
    rendering::{
        buffer_manager::{dupe, dupe_static},
        material::MaterialGeneric,
        mesh::mesh::TextureVertex,
        render_entity::render_entity::RawTextureMaterialInstance,
    },
    ENGINE_RUNTIME,
};

use super::RenderManagerResource;

impl<'w: 'static> EngineRuntime<'w> {
    pub fn render(&'w mut self) -> Result<(), SurfaceError> {
        let state = dupe(self).get_resource::<RenderManagerResource>();
        let output = state.surface.get_current_texture()?;

        let view = output
            .texture
            .create_view(&TextureViewDescriptor::default());

        let mut encoder = state
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Render encoder"),
            });

        //let pipeline = dupe(state).material.render_pipeline(dupe(self));

        std::fs::read_to_string("resources/shaders/shader_texture.wgsl").expect("failed");
        let file =
            std::fs::read_to_string("resources/shaders/shader_texture.wgsl").expect("failed");
        let shader = state.device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Shader"),
            source: ShaderSource::Wgsl(file.as_str().into()),
        });

        let camera_bind_group_layout =
            state
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
            state
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
            state
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &[
                        &crate::rendering::texture::texture::Texture::get_texture_bind_layout(
                            &state.device,
                        ),
                        &camera_bind_group_layout,
                        &aspect_bind_group_layout,
                    ],
                    push_constant_ranges: &[],
                });

        {
            // 'pass is tied to encoder
            let mut render_pass = encoder.begin_render_pass(
                //and 'tex in desc is eq to 'pass
                &RenderPassDescriptor {
                    label: Some("Render Pass"),
                    color_attachments: &[Some(RenderPassColorAttachment {
                        view: &view, // 'tex eq 'tex
                        resolve_target: None,
                        ops: Operations {
                            load: LoadOp::Clear(*state.clear_color),
                            store: StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                        view: &state.depth_texture.view,
                        depth_ops: Some(Operations {
                            load: LoadOp::Clear(1.),
                            store: StoreOp::Store,
                        }),
                        stencil_ops: None,
                    }),
                    occlusion_query_set: None,
                    timestamp_writes: None,
                },
            );

            let pipeline = state
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
                            format: state.config.format,
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
                        format: crate::rendering::texture::texture::Texture::DEPTH_FORMAT,
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

            render_pass.set_pipeline(&pipeline); // render pass has 'a which is eq ot encoders 'pass

            state.material.set_bind_groups(&mut render_pass, dupe(self));

            render_pass.set_vertex_buffer(0, state.material.vertex_buffer(dupe(self)).slice(..));
            render_pass.set_index_buffer(
                state.material.index_buffer(dupe(self)).slice(..),
                IndexFormat::Uint16,
            );
            // set instance buffer
            render_pass.set_vertex_buffer(1, state.material.instance_buffer(dupe(self)).slice(..));

            render_pass.draw_indexed(
                state.material.get_index_range(),
                0,
                state.material.get_instance_range(),
            );
            drop(render_pass);
        }
        // submit will accept anything that implements IntoIter
        state.queue.submit(std::iter::once(encoder.finish())); // only thing that borrowed encoder was the render pass which was dropped in the previous scope but rustc doesnt see it
        output.present();

        Ok(())
    }
}
