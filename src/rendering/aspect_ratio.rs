use winit::dpi::PhysicalSize;

#[repr(C)]
// This is so we can store this in a buffer
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct AspectUniform {
    pub aspect_ratio: f32,
    pub width: f32,
    pub height: f32,
}
impl Default for AspectUniform {
    fn default() -> Self {
        Self::new()
    }
}

impl AspectUniform {
    pub fn new() -> Self {
        AspectUniform {
            aspect_ratio: 1.,
            width: 100.,
            height: 100.,
        }
    }
    pub fn from_size(size: PhysicalSize<u32>) -> Self {
        Self {
            aspect_ratio: size.width as f32 / size.height as f32,
            width: size.width as f32,
            height: size.height as f32,
        }
    }
}
