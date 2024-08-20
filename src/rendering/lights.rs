use bevy_ecs::component::Component;
use rapier3d::na::UnitQuaternion;

use crate::{marker_comps, ENGINE_RUNTIME};

use super::{
    camera::camera::Camera,
    systems::general::{Transform, Translation, Vector},
};

marker_comps!(LightLookDirText);

#[derive(Component, Default)]
pub struct PointLight {
    pub light_strenght: f32,
    pub ambient_strenght: f32,
}

impl PointLight {
    pub fn new(light_strenght: f32) -> Self {
        Self {
            light_strenght,
            ambient_strenght: 0.03,
        }
    }
    pub fn new_with_ambient(light_strenght: f32, ambient_strenght: f32) -> Self {
        Self {
            light_strenght,
            ambient_strenght,
        }
    }
}
#[derive(Component)]
pub struct SpotLight {
    pub light_strenght: f32,
    pub ambient_strenght: f32,
    pub inner_angle: f32,
    pub outer_angle: f32,
}

impl SpotLight {
    pub fn new(light_strenght: f32, inner_angle: f32, outer_angle: f32) -> Self {
        Self {
            light_strenght,
            ambient_strenght: 0.03,
            inner_angle,
            outer_angle,
        }
    }
    pub fn new_with_ambient(
        light_strenght: f32,
        ambient_strenght: f32,
        inner_angle: f32,
        outer_angle: f32,
    ) -> Self {
        Self {
            light_strenght,
            ambient_strenght,
            inner_angle,
            outer_angle,
        }
    }
}

#[derive(Default, Debug)]
pub struct PointLightType {
    pub position: Translation,
    pub color: wgpu::Color,
    pub light_strenght: f32,
    pub ambient_strenght: f32,
}
impl PointLightType {
    pub fn new(
        position: Translation,
        color: wgpu::Color,
        light_strenght: f32,
        ambient_strenght: f32,
    ) -> Self {
        Self {
            position,
            color,
            light_strenght,
            ambient_strenght,
        }
    }

    pub fn to_raw(&self) -> PointLightUniform {
        PointLightUniform::new(
            self.position.into(),
            {
                [
                    self.color.r as f32,
                    self.color.g as f32,
                    self.color.b as f32,
                    self.color.a as f32,
                ]
            },
            self.light_strenght,
            self.ambient_strenght,
        )
    }
}

#[repr(C)]
#[derive(Default, Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PointLightUniform {
    pub position: [f32; 4],
    pub color: [f32; 4],
    pub light_strenght: f32,
    pub ambient_strenght: f32,
    _p1: f32,
    _p2: f32,
}

impl PointLightUniform {
    pub fn new(
        position: [f32; 3],
        color: [f32; 4],
        light_strenght: f32,
        ambient_strenght: f32,
    ) -> Self {
        Self {
            position: [position[0], position[1], position[2], 0.],
            color: [color[0], color[1], color[2], color[3]],
            light_strenght,
            ambient_strenght,
            _p1: 0.,
            _p2: 0.,
        }
    }
}

#[derive(Default, Debug)]
pub struct SpotLightType {
    pub transform: Transform,
    pub color: wgpu::Color,
    pub light_strenght: f32,
    pub ambient_strenght: f32,
    pub inner_angle: f32,
    pub outer_angle: f32,
}
impl SpotLightType {
    pub fn new(
        transform: Transform,
        color: wgpu::Color,
        light_strenght: f32,
        ambient_strenght: f32,
        inner_angle: f32,
        outer_angle: f32,
    ) -> Self {
        Self {
            transform,
            color,
            light_strenght,
            ambient_strenght,
            inner_angle,
            outer_angle,
        }
    }

    pub fn to_raw(&self) -> SpotLightUniform {
        let engine = unsafe { ENGINE_RUNTIME.get() };
        let camera = engine
            .ecs
            .world
            .query::<&Camera>()
            .get_single(&engine.ecs.world)
            .unwrap();

        SpotLightUniform::new(
            self.transform.translation.into(),
            {
                [
                    self.color.r as f32,
                    self.color.g as f32,
                    self.color.b as f32,
                    self.color.a as f32,
                ]
            },
            ((camera.rot_matrix * -Vector::z()).normalize() * -1.).into(),
            self.light_strenght,
            self.ambient_strenght,
            self.inner_angle,
            self.outer_angle,
        )
    }
}
pub trait QuatExt {
    fn get_forward(&self) -> Vector;
}
impl QuatExt for UnitQuaternion<f32> {
    fn get_forward(&self) -> Vector {
        let mut forward = self.inverse() * (self * Vector::z());
        forward.normalize_mut();
        forward
    }
}

#[repr(C)]
#[derive(Default, Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SpotLightUniform {
    pub position: [f32; 4],
    pub color: [f32; 4],
    pub direction: [f32; 4],
    pub light_strenght: f32,
    pub ambient_strenght: f32,
    pub inner_angle: f32,
    pub outer_angle: f32,
    _p: [f32; 4],
}

impl SpotLightUniform {
    pub fn new(
        position: [f32; 3],
        color: [f32; 4],
        direction: [f32; 3],
        light_strenght: f32,
        ambient_strenght: f32,
        inner_angle: f32,
        outer_angle: f32,
    ) -> Self {
        Self {
            position: [position[0], position[1], position[2], 0.],
            color: [color[0], color[1], color[2], color[3]],
            direction: [direction[0], direction[1], direction[2], 0.],
            light_strenght,
            ambient_strenght,
            inner_angle,
            outer_angle,
            _p: Default::default(),
        }
    }
}
#[repr(C)]
#[derive(Default, Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct IndexUniform {
    index: u32,
    _p1: u32,
    _p2: u32,
    _p3: u32,
}

impl IndexUniform {
    pub fn new(index: u32) -> Self {
        Self {
            index,
            _p1: 0,
            _p2: 0,
            _p3: 0,
        }
    }
}
