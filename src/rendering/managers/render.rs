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

        span!(trace_set_up_materials, "setting up materials");
        let light_material = &mut state.light_material.deref_mut();
        let texture_material = &mut state.texture_material.deref_mut();

        light_material.setup_bind_groups(self);
        texture_material.setup_bind_groups(self);

        span!(trace_creating_render_pipelines, "creating pipelines");

        let pipeline_light = dupe(light_material).render_pipeline(self);

        let pipeline_texture = dupe(texture_material).render_pipeline(self);

        drop_span!(trace_creating_render_pipelines);
        drop_span!(trace_set_up_materials);

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
        span!(trace_light_material, "light_material");

        render_pass.set_pipeline(&pipeline_light);

        light_material.set_bind_groups(&mut render_pass, self);

        render_pass.set_vertex_buffer(0, light_material.vertex_buffer(self).slice(..));
        render_pass.set_index_buffer(
            light_material.index_buffer(self).slice(..),
            IndexFormat::Uint16,
        );
        // set instance buffer
        render_pass.set_vertex_buffer(1, light_material.instance_buffer(self).slice(..));

        render_pass.draw_indexed(
            light_material.get_index_range(),
            0,
            light_material.get_instance_range(),
        );
        drop_span!(trace_light_material);

        // TEXTURE MATERIAL

        span!(trace_texture_material, "texture_material");

        render_pass.set_pipeline(&pipeline_texture);
        texture_material.set_bind_groups(&mut render_pass, self);
        render_pass.set_vertex_buffer(0, texture_material.vertex_buffer(self).slice(..));

        render_pass.set_index_buffer(
            texture_material.index_buffer(self).slice(..),
            IndexFormat::Uint16,
        );
        // set instance buffer
        render_pass.set_vertex_buffer(1, texture_material.instance_buffer(self).slice(..));

        render_pass.draw_indexed(
            texture_material.get_index_range(),
            0,
            texture_material.get_instance_range(),
        );

        drop_span!(trace_texture_material);
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
