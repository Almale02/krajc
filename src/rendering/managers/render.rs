use std::ops::DerefMut;

use wgpu::*;

use crate::{
    engine_runtime::EngineRuntime, rendering::material::MaterialGeneric, typed_addr::dupe,
    ENGINE_RUNTIME,
};

use super::RenderManagerResource;

impl EngineRuntime {
    pub fn render(&mut self) -> Result<(), SurfaceError> {
        let state = self.get_resource_mut::<RenderManagerResource>();
        let output = state.surface.get_current_texture()?;

        let view = output
            .texture
            .create_view(&TextureViewDescriptor::default());

        let mut encoder = state
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Render encoder"),
            });

        let light_material = &mut state.light_material.deref_mut();

        light_material.setup_bind_groups(self);

        let pipeline = light_material.render_pipeline(self);
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

        state.light_material.set_bind_groups(&mut render_pass, self);

        render_pass.set_vertex_buffer(0, state.light_material.vertex_buffer(self).slice(..));
        render_pass.set_index_buffer(
            state.light_material.index_buffer(self).slice(..),
            IndexFormat::Uint16,
        );
        // set instance buffer
        render_pass.set_vertex_buffer(1, state.light_material.instance_buffer(self).slice(..));

        render_pass.draw_indexed(
            state.light_material.get_index_range(),
            0,
            state.light_material.get_instance_range(),
        );

        render_pass.set_vertex_buffer(0, state.light_material.vertex_buffer(self).slice(..));
        render_pass.set_index_buffer(
            state.light_material.index_buffer(self).slice(..),
            IndexFormat::Uint16,
        );
        // set instance buffer
        render_pass.set_vertex_buffer(1, state.light_material.instance_buffer(self).slice(..));

        render_pass.draw_indexed(
            state.light_material.get_index_range(),
            0,
            state.light_material.get_instance_range(),
        );

        drop(render_pass);

        // submit will accept anything that implements IntoIter
        state.queue.submit(vec![encoder.finish()]);
        output.present();

        Ok(())
    }
}
