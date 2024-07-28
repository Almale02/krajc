use bevy_ecs::component::Component;
use bytemuck::{Pod, Zeroable};
use cgmath::{prelude::*, *};

use crate::*;

#[derive(Clone, PartialEq, Debug, Component)]
pub struct LightMaterialInstance {
    pub position: Vec3,
    pub rotation: Quaternion<f32>,
}

impl Default for LightMaterialInstance {
    fn default() -> Self {
        Self {
            position: Vec3::new(0., 0., 0.),
            rotation: <Quaternion<f32>>::zero(),
        }
    }
}
impl LightMaterialInstance {
    pub fn new(position: Vec3, rotation: Quaternion<f32>) -> Self {
        Self { position, rotation }
    }
    pub fn from_pos(pos: Vec3) -> Self {
        Self::new(pos, Quaternion::zero())
    }
    pub fn to_raw(&self) -> RawLightMaterialInstance {
        let model =
            Matrix4::from_translation(self.position.as_vector3()) * Matrix4::from(self.rotation);

        RawLightMaterialInstance {
            model: std::convert::Into::into(model),
            normal: Matrix3::from(self.rotation).into(),
        }
    }
}
impl From<LightMaterialInstance> for RawLightMaterialInstance {
    fn from(val: LightMaterialInstance) -> Self {
        val.to_raw()
    }
}

#[repr(C)]
#[derive(Clone, Copy, Default, Pod, Zeroable)]
pub struct RawLightMaterialInstance {
    pub model: [[f32; 4]; 4],
    pub normal: [[f32; 3]; 3],
}

impl RawLightMaterialInstance {
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
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 9,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 10,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 11,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}
