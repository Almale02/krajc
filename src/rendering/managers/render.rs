use std::ops::DerefMut;

use wgpu::*;

use crate::{
    drop_span, engine_runtime::EngineRuntime, rendering::material::MaterialGeneric, span,
    typed_addr::dupe, ENGINE_RUNTIME,
};

use super::RenderManagerResource;

impl EngineRuntime {
    pub fn render(&mut self) -> Result<(), SurfaceError> {
        let state = self.get_resource_mut::<RenderManagerResource>();

        span!(trace_get_surface_texture, "get surface texture");
        let output = state.surface.get_current_texture()?;
        drop_span!(trace_get_surface_texture);

        let view = output
            .texture
            .create_view(&TextureViewDescriptor::default());

        let mut encoder = state
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Render encoder"),
            });

        span!(trace_render_pass, "render pass"); // before this most of the time
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

        for draw_pass in state.draw_passes.iter_mut() {
            if draw_pass.is_loaded() {
                draw_pass.draw(&mut render_pass, dupe(self));
            }
        }
        span!(trace_light_material, "light_material");

        drop_span!(trace_render_pass);
        drop(render_pass);

        span!(trace_queue, "queue submit");
        state.queue.submit(vec![encoder.finish()]);
        drop_span!(trace_queue);

        span!(trace_present, "present");
        output.present();
        drop_span!(trace_present);

        Ok(())
    }
}
