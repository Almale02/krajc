use bevy_ecs::component::Component;
use cgmath::{perspective, InnerSpace, Matrix4, Rad};
//use cgmath::*;
use rapier3d::na::UnitQuaternion;
use std::f32::consts::FRAC_PI_2;
use winit::{
    dpi::PhysicalPosition,
    event::{ElementState, MouseScrollDelta, VirtualKeyCode},
};

use crate::rendering::systems::general::Transform;

pub const SAFE_FRAC_PI_2: f32 = FRAC_PI_2 - 0.0001;

#[rustfmt::skip]
    pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
        1.0, 0.0, 0.0, 0.0,
        0.0, 1.0, 0.0, 0.0,
        0.0, 0.0, 0.5, 0.5,
        0.0, 0.0, 0.0, 1.0,
    );

#[derive(Default, Component)]
pub struct Camera;

impl Camera {
    pub fn calc_matrix(iso: &mut Transform) -> Matrix4<f32> {
        let (sin_pitch, cos_pitch) = iso.rotation.euler_angles().1.sin_cos();
        let (sin_yaw, cos_yaw) = iso.rotation.euler_angles().2.sin_cos();

        Matrix4::look_to_rh(
            cgmath::Point3 {
                x: iso.translation.x,
                y: iso.translation.y,
                z: iso.translation.z,
            },
            cgmath::Vector3::new(cos_pitch * cos_yaw, sin_pitch, cos_pitch * sin_yaw).normalize(),
            cgmath::Vector3::unit_y(),
        )
    }
}

pub struct Projection {
    aspect: f32,
    fovy: Rad<f32>,
    znear: f32,
    zfar: f32,
}

impl Projection {
    pub fn new<F: Into<Rad<f32>>>(width: u32, height: u32, fovy: F, znear: f32, zfar: f32) -> Self {
        Self {
            aspect: width as f32 / height as f32,
            fovy: fovy.into(),
            znear,
            zfar,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height as f32;
    }

    pub fn calc_matrix(&self) -> Matrix4<f32> {
        OPENGL_TO_WGPU_MATRIX * perspective(self.fovy, self.aspect, self.znear, self.zfar)
    }
}

#[repr(C)]
// This is so we can store this in a buffer
#[derive(Default, Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]

pub struct CameraUniform {
    pub view_proj: [[f32; 4]; 4],
    pub view_pos: [f32; 4],
}

impl CameraUniform {
    pub fn update_view_proj(&mut self, iso: &mut Transform, projection: &Projection) {
        let pos = cgmath::Point3::new(iso.translation.x, iso.translation.y, iso.translation.z);
        //self.view_pos = pos.to_homogeneous().into();
        self.view_proj = (projection.calc_matrix() * Camera::calc_matrix(iso)).into();
    }
}

#[derive(Debug)]
pub struct CameraController {
    sprinting: bool,
    base_speed: f32,
    amount_left: f32,
    amount_right: f32,
    amount_forward: f32,
    amount_backward: f32,
    amount_up: f32,
    amount_down: f32,
    rotate_horizontal: f32,
    rotate_vertical: f32,
    scroll: f32,
    speed: f32,
    sensitivity: f32,
}

impl CameraController {
    pub fn new(speed: f32, sensitivity: f32) -> Self {
        Self {
            sprinting: false,
            base_speed: speed,
            amount_left: 0.0,
            amount_right: 0.0,
            amount_forward: 0.0,
            amount_backward: 0.0,
            amount_up: 0.0,
            amount_down: 0.0,
            rotate_horizontal: 0.0,
            rotate_vertical: 0.0,
            scroll: 0.0,
            speed,
            sensitivity,
        }
    }

    pub fn process_keyboard(&mut self, key: VirtualKeyCode, state: ElementState) -> bool {
        let amount = if state == ElementState::Pressed {
            1.0
        } else {
            0.0
        };
        if key == VirtualKeyCode::LShift && state == ElementState::Pressed {
            self.sprinting = !self.sprinting;
        }
        match key {
            VirtualKeyCode::W | VirtualKeyCode::Up => {
                self.amount_forward = amount;
                true
            }
            VirtualKeyCode::S | VirtualKeyCode::Down => {
                self.amount_backward = amount;
                true
            }
            VirtualKeyCode::A | VirtualKeyCode::Left => {
                self.amount_left = amount;
                true
            }
            VirtualKeyCode::D | VirtualKeyCode::Right => {
                self.amount_right = amount;
                true
            }
            VirtualKeyCode::Space => {
                self.amount_up = amount;
                true
            }
            VirtualKeyCode::LControl => {
                self.amount_down = amount;
                true
            }
            _ => false,
        }
    }

    pub fn process_mouse(&mut self, mouse_dx: f64, mouse_dy: f64) {
        self.rotate_horizontal = mouse_dx as f32;
        self.rotate_vertical = mouse_dy as f32;
    }

    pub fn process_scroll(&mut self, delta: &MouseScrollDelta) {
        self.scroll = -match delta {
            // I'm assuming a line is about 100 pixels
            MouseScrollDelta::LineDelta(_, scroll) => scroll * 100.0,
            MouseScrollDelta::PixelDelta(PhysicalPosition { y: scroll, .. }) => *scroll as f32,
        };
    }

    pub fn update_camera(&mut self, iso: &mut Transform, dt: f64) {
        let mut yaw = iso.rotation.euler_angles().2;
        let mut pitch = iso.rotation.euler_angles().1;

        let dt = dt as f32;
        if self.sprinting {
            self.speed = self.base_speed * 3.
        } else {
            self.speed = self.base_speed
        }
        let mut cam_pos =
            cgmath::Vector3::new(iso.translation.x, iso.translation.y, iso.translation.z);

        // Move forward/backward and left/right
        let (yaw_sin, yaw_cos) = yaw.sin_cos();
        let forward = cgmath::Vector3::new(yaw_cos, 0.0, yaw_sin).normalize();
        let right = cgmath::Vector3::new(-yaw_sin, 0.0, yaw_cos).normalize();
        cam_pos +=
            forward * ((self.amount_forward - self.amount_backward) * self.speed * dt).into();
        cam_pos += right * (self.amount_right - self.amount_left) * self.speed * dt;

        // Move in/out (aka. "zoom")
        // Note: this isn't an actual zoom. The camera's position
        // changes when zooming. I've added this to make it easier
        // to get closer to an object you want to focus on.
        let (pitch_sin, pitch_cos) = pitch.sin_cos();
        let scrollward =
            cgmath::Vector3::new(pitch_cos * yaw_cos, pitch_sin, pitch_cos * yaw_sin).normalize();
        cam_pos += scrollward * self.scroll * self.speed * self.sensitivity * dt;
        self.scroll = 0.0;

        // Move up/down. Since we don't use roll, we can just
        // modify the y coordinate directly.
        cam_pos.y += (self.amount_up - self.amount_down) * self.speed * dt;

        // Rotate
        yaw += (Rad(self.rotate_horizontal) * self.sensitivity * dt).0;
        pitch += (Rad(-self.rotate_vertical) * self.sensitivity * dt).0;

        // If process_mouse isn't called every frame, these values
        // will not get set to zero, and the camera will rotate
        // when moving in a non-cardinal direction.
        self.rotate_horizontal = 0.0;
        self.rotate_vertical = 0.0;

        // Keep the camera's angle from going too high/low.
        if pitch < -Rad(SAFE_FRAC_PI_2).0 {
            pitch = -Rad(SAFE_FRAC_PI_2).0;
        } else if pitch > Rad(SAFE_FRAC_PI_2).0 {
            pitch = Rad(SAFE_FRAC_PI_2).0;
        }
        iso.translation.x = cam_pos.x;
        iso.translation.y = cam_pos.y;
        iso.translation.z = cam_pos.z;

        iso.rotation = UnitQuaternion::from_euler_angles(0., pitch, yaw);
    }
}
