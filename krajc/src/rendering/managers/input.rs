use winit::event::{KeyboardInput, MouseButton, WindowEvent};

use crate::engine_runtime::EngineRuntime;

use super::RenderManagerResource;

impl EngineRuntime {
    #[allow(deprecated, unreachable_patterns)]
    pub fn window_events(&mut self, event: &WindowEvent) -> bool {
        let render_state = self.get_resource_mut::<RenderManagerResource>();
        match event {
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        virtual_keycode: Some(key),
                        state,
                        ..
                    },
                ..
            } => render_state
                .camera_controller
                .process_keyboard(*key, *state),

            WindowEvent::MouseWheel { delta, .. } => {
                render_state.camera_controller.process_scroll(delta);
                true
            }

            WindowEvent::MouseInput {
                button: MouseButton::Left,
                state: _,
                ..
            } => true,
            _ => false,

            WindowEvent::CursorMoved {
                device_id: _,
                position,
                modifiers: _,
            } => {
                let mut clear_color = *render_state.clear_color;
                let config = &*render_state.config;
                clear_color.r = position.x / config.width as f64;
                clear_color.b = position.y / config.height as f64;
                true
            }
            _ => false,
        }
    }
}
