use crate::{engine_runtime::EngineRuntime, rendering::texture::texture::Texture};

use super::RenderManagerResource;

impl EngineRuntime {
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        let state = self.get_resource_mut::<RenderManagerResource>();
        if new_size.width > 0 && new_size.height > 0 {
            *state.size = new_size;
            state.config.width = new_size.width;
            state.config.height = new_size.height;
            state.projection.resize(new_size.width, new_size.height);
            state.surface.configure(&state.device, &state.config);

            *state.depth_texture =
                Texture::create_depth_texture(&state.device, &state.config, "Depth Texture");
        }
    }
}
