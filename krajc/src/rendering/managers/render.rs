use glyphon::{Resolution, TextArea, TextBounds};
use wgpu::*;

use crate::{engine_runtime::EngineRuntime, typed_addr::dupe};
//use crate::span;
//use crate::drop_span;

use super::RenderManagerResource;

impl EngineRuntime {
    pub fn render(&mut self) -> Result<(), SurfaceError> {
        let state = self.get_resource_mut::<RenderManagerResource>();

        dupe(&state.text_state)
            .text_render
            .prepare(
                &state.device,
                &state.queue,
                dupe(&state.text_state).font_system,
                dupe(&state.text_state).atlas,
                Resolution {
                    width: state.size.width,
                    height: state.size.height,
                },
                [TextArea {
                    buffer: &dupe(&state.text_state).buffer,
                    left: 10.0,
                    top: 10.0,
                    scale: 1.0,
                    bounds: TextBounds {
                        left: 0,
                        top: 0,
                        right: 600,
                        bottom: 160,
                    },
                    default_color: glyphon::Color::rgb(255, 255, 255),
                }],
                &mut state.text_state.cahce,
            )
            .unwrap();

        //span!(trace_get_surface_texture, "get surface texture");
        let output = state.surface.get_current_texture()?;
        //drop_span!(trace_get_surface_texture);

        let view = output
            .texture
            .create_view(&TextureViewDescriptor::default());

        let mut encoder = state
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Render encoder"),
            });

        //span!(trace_render_pass, "render pass"); // before this most of the time
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

        for draw_pass in dupe(state).draw_passes.iter_mut() {
            if draw_pass.is_loaded(self) {
                draw_pass.draw(&mut render_pass, dupe(self));
            }
        }

        drop(render_pass);
        //span!(trace_render_pass, "render pass"); // before this most of the time
        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Load,
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        state
            .text_state
            .text_render
            .render(state.text_state.atlas, &mut render_pass)
            .unwrap();
        drop(render_pass);

        state.draw_passes.clear();
        //span!(trace_light_material, "light_material");

        //drop_span!(trace_render_pass);
        //drop(render_pass);

        //span!(trace_queue, "queue submit");
        state.queue.submit(vec![encoder.finish()]);
        //drop_span!(trace_queue);

        //span!(trace_present, "present");
        output.present();
        //drop_span!(trace_present);

        state.text_state.atlas.trim();

        Ok(())
    }
}
