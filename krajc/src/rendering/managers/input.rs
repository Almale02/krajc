use winit::event::{MouseButton, WindowEvent};

use crate::engine_runtime::EngineRuntime;

use super::RenderManagerResource;

impl EngineRuntime {
    #[allow(deprecated, unreachable_patterns)]
    pub fn window_events(&mut self, event: &WindowEvent) {
        let render_state = self.get_resource_mut::<RenderManagerResource>();
        match event {
            WindowEvent::KeyboardInput { event, .. } => match event.physical_key {
                winit::keyboard::PhysicalKey::Code(x) => {
                    render_state
                        .camera_controller
                        .process_keyboard(x, event.state);
                }
                winit::keyboard::PhysicalKey::Unidentified(_) => (),
            },

            WindowEvent::MouseWheel { delta, .. } => {
                render_state.camera_controller.process_scroll(delta);
            }

            WindowEvent::MouseInput {
                button: MouseButton::Left,
                state: _,
                ..
            } => (),
            _ => (),

            WindowEvent::CursorMoved {
                device_id: _,
                position,
            } => {
                let mut clear_color = *render_state.clear_color;
                let config = &*render_state.config;
                clear_color.r = position.x / config.width as f64;
                clear_color.b = position.y / config.height as f64;
            }
            _ => (),
        }
    }
}
