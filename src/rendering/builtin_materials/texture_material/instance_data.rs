use bevy_ecs::component::Component;
use bytemuck::{Pod, Zeroable};
use rapier3d::na::Matrix4;

use crate::*;

#[derive(Clone, PartialEq, Debug, Component)]
pub struct TextureMaterialInstance {
    pub transform: Transform,
}

impl Default for TextureMaterialInstance {
    fn default() -> Self {
        Self {
            transform: Transform::default(),
        }
    }
}
impl TextureMaterialInstance {
    pub fn new(transform: Transform) -> Self {
        Self { transform }
    }
    pub fn to_raw(&self) -> RawTextureMaterialInstance {
        let engine = unsafe { ENGINE_RUNTIME.get() };

        //let translation_matrix = Translation3::from(self.transform.translation);
        let mut model_matrix = Matrix4::identity();

        let rot_matrix = self
            .transform
            .rotation
            .to_rotation_matrix()
            .to_homogeneous();

        model_matrix.fixed_view_mut::<4, 4>(0, 0).copy_from(
            &rot_matrix, /*
                         &engine
                             .ecs
                             .world
                             .query::<&Camera>()
                             .single(&engine.ecs.world)
                             .rot_matrix,*/
        );

        model_matrix[(0, 3)] = self.transform.translation.vector.x;
        model_matrix[(1, 3)] = self.transform.translation.vector.y;
        model_matrix[(2, 3)] = self.transform.translation.vector.z; //let isometry = Isometry3::from_parts(translation_matrix, self.transform.rotation);

        RawTextureMaterialInstance {
            model: model_matrix.into(),
        }
    }
}
impl From<TextureMaterialInstance> for RawTextureMaterialInstance {
    fn from(val: TextureMaterialInstance) -> Self {
        val.to_raw()
    }
}

#[repr(C)]
#[derive(Clone, Copy, Default, Pod, Zeroable)]
pub struct RawTextureMaterialInstance {
    pub model: [[f32; 4]; 4],
}

impl RawTextureMaterialInstance {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            // We need to switch from using a step mode of Vertex to Instance
            // This means that our shaders will only change to use the next
            // instance when the shader starts processing a new instance
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                // A mat4 takes up 4 vertex slots as it is technically 4 vec4s. We need to define a slot
                // for each vec4. We'll have to reassemble the mat4 in the shader.
                wgpu::VertexAttribute {
                    offset: 0,
                    // While our vertex shader only uses locations 0, and 1 now, in later tutorials, we'll
                    // be using 2, 3, and 4, for Vertex. We'll start at slot 5, not conflict with them later
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}
