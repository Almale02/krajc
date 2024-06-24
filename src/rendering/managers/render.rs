use wgpu::*;

use crate::{
    engine_runtime::EngineRuntime,
    rendering::{material::MaterialGeneric},
    ENGINE_RUNTIME,
};

use super::RenderManagerResource;

impl EngineRuntime {
    pub fn render(&mut self) -> Result<(), SurfaceError> {
        let engine = unsafe { ENGINE_RUNTIME.get() };
        let state = unsafe { ENGINE_RUNTIME.get().get_resource::<RenderManagerResource>() };
        let output = state.surface.get_current_texture()?;

        let view = output
            .texture
            .create_view(&TextureViewDescriptor::default());

        let mut encoder = state
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Render encoder"),
            });

        let pipeline = state
            .material
            .render_pipeline(unsafe { ENGINE_RUNTIME.get() });

        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &view,
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
        });
        render_pass.set_pipeline(&pipeline);

        state.material.set_bind_groups(&mut render_pass, engine);

        render_pass.set_vertex_buffer(0, state.material.vertex_buffer(engine).slice(..));
        render_pass.set_index_buffer(
            state.material.index_buffer(engine).slice(..),
            IndexFormat::Uint16,
        );
        // set instance buffer
        render_pass.set_vertex_buffer(1, state.material.instance_buffer(engine).slice(..));

        render_pass.draw_indexed(
            state.material.get_index_range(),
            0,
            0..state.instance_scheme.len() as u32,
        );

        drop(render_pass);

        // submit will accept anything that implements IntoIter
        state.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
