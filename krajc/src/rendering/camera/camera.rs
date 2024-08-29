use crate::{rendering::systems::general::Vector, UnitQuaternion};
use bevy_ecs::component::Component;
use rapier3d::na::{Isometry3, Matrix3, Matrix4, Point3, Rotation3};
use std::f32::consts::FRAC_PI_2;
use winit::{
    dpi::PhysicalPosition,
    event::{ElementState, MouseScrollDelta }, keyboard::KeyCode,
};

use crate::rendering::systems::general::Transform;

pub const SAFE_FRAC_PI_2: f32 = FRAC_PI_2 - 0.0001;

#[rustfmt::skip]
    pub const OPENGL_TO_WGPU_MATRIX: Matrix4<f32> = Matrix4::new(
        1.0, 0.0, 0.0, 0.0,
        0.0, 1.0, 0.0, 0.0,
        0.0, 0.0, 0.5, 0.5,
        0.0, 0.0, 0.0, 1.0,
    );

#[derive(Default, Component)]
pub struct Camera {
    pub forward: Vector,
    pub up: Vector,
    pub right: Vector,
    pub rot_matrix: Matrix3<f32>,
    pub quat: UnitQuaternion<f32>,
}

impl Camera {
    pub fn new() -> Self {
        Self::default()
    }

    #[rustfmt::skip]
    pub fn calc_view_matrix(iso: &mut Transform) -> Matrix4<f32> {

        let (_roll, pitch, yaw) = iso.rotation.euler_angles();

        // Compute sin and cos for pitch and yaw
        let (sin_pitch, cos_pitch) = pitch.sin_cos();
        let (sin_yaw, cos_yaw) = yaw.sin_cos();

        // Create rotation matrices
        let rotation_yaw = Matrix3::new(
            cos_yaw, 0.0, -sin_yaw,
            0.0, 1.0, 0.0,
            sin_yaw, 0.0, cos_yaw,
        );

        let rotation_pitch = Matrix3::new(
            1.0, 0.0, 0.0,
            0.0, cos_pitch, sin_pitch, // Inverted sign for pitch
            0.0, -sin_pitch, cos_pitch, // Inverted sign for pitch
        );

        // Combine yaw and pitch rotations (yaw first, then pitch)
        let rotation_matrix = rotation_yaw * rotation_pitch;


        let forward = rotation_matrix * (-Vector::z());

        
        let target = iso.translation.vector + forward;
        

        Isometry3::look_at_rh(
            &Point3::from(iso.translation.vector),
            &Point3::from(
                target
            ),
            &Vector::y(),
        )
        .to_homogeneous()
    }
}

pub struct Projection {
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,
}

impl Projection {
    pub fn new(width: u32, height: u32, fovy: f32, znear: f32, zfar: f32) -> Self {
        Self {
            aspect: width as f32 / height as f32,
            fovy,
            znear,
            zfar,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height as f32;
    }

    pub fn projection_matrix(&self) -> Matrix4<f32> {
        //rapier3d::na::Perspective3::new(, , , )

        OPENGL_TO_WGPU_MATRIX
            * Matrix4::new_perspective(self.aspect, self.fovy, self.znear, self.zfar)
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
        self.view_proj = (projection.projection_matrix() * Camera::calc_view_matrix(iso)).into();
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

    pub fn process_keyboard(&mut self, key: KeyCode, state: ElementState) -> bool {
        let amount = if state == ElementState::Pressed {
            1.0
        } else {
            0.0
        };
        if key == KeyCode::ShiftLeft && state == ElementState::Pressed {
            self.sprinting = !self.sprinting;
        }
        match key {
            KeyCode::KeyW | KeyCode::ArrowUp => {
                self.amount_forward = amount;
                true
            }
            KeyCode::KeyS | KeyCode::ArrowDown => {
                self.amount_backward = amount;
                true
            }
            KeyCode::KeyA | KeyCode::ArrowLeft => {
                self.amount_left = amount;
                true
            }
            KeyCode::KeyD | KeyCode::ArrowRight => {
                self.amount_right = amount;
                true
            }
            KeyCode::Space => {
                self.amount_up = amount;
                true
            }
            KeyCode::ControlLeft => {
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

    pub fn update_camera(
        &mut self,
        iso: &mut Transform,
        dt: f64,
        camera: &mut Camera,
        camera_rot_text: &mut String,
    ) {
        let mut pitch = iso.rotation.euler_angles().1;
        let mut yaw = iso.rotation.euler_angles().2;

        // Compute sin and cos for pitch and yaw
        let (sin_pitch, cos_pitch) = pitch.sin_cos();
        let (sin_yaw, cos_yaw) = yaw.sin_cos();

        // Create rotation matrices
        let rotation_yaw =
            Matrix3::new(cos_yaw, 0.0, -sin_yaw, 0.0, 1.0, 0.0, sin_yaw, 0.0, cos_yaw);

        let rotation_pitch = Matrix3::new(
            1.0, 0.0, 0.0, 0.0, cos_pitch, sin_pitch, 0.0, -sin_pitch, cos_pitch,
        );

        // Combine yaw and pitch rotations (yaw first, then pitch)
        let rotation_matrix = rotation_yaw * rotation_pitch;
        let rotation_quat =
            UnitQuaternion::from_rotation_matrix(&Rotation3::from_matrix(&rotation_matrix));

        let forward = rotation_quat * (-Vector::z());
        let forward_no_up = Vector::new(forward.x, 0., forward.z).normalize();
        let right = rotation_quat * Vector::x();

        let dt = dt as f32;
        if self.sprinting {
            self.speed = self.base_speed * 3.
        } else {
            self.speed = self.base_speed
        }
        let mut cam_pos = Vector::new(iso.translation.x, iso.translation.y, iso.translation.z);

        // Move forward/backward and left/right
        let (yaw_sin, yaw_cos) = yaw.sin_cos();
        cam_pos += forward_no_up * ((self.amount_forward - self.amount_backward) * self.speed * dt);
        cam_pos += right * (self.amount_right - self.amount_left) * self.speed * dt;

        // Move in/out (aka. "zoom")
        // Note: this isn't an actual zoom. The camera's position
        // changes when zooming. I've added this to make it easier
        // to get closer to an object you want to focus on.
        let (pitch_sin, pitch_cos) = pitch.sin_cos();
        let scrollward =
            Vector::new(pitch_cos * yaw_cos, pitch_sin, pitch_cos * yaw_sin).normalize();
        cam_pos += scrollward * self.scroll * self.speed * self.sensitivity * dt;
        self.scroll = 0.0;

        // Move up/down. Since we don't use roll, we can just
        // modify the y coordinate directly.
        cam_pos.y += (self.amount_up - self.amount_down) * self.speed * dt;

        // Rotate
        yaw += (self.rotate_horizontal) * self.sensitivity * dt * 0.4;
        pitch += (self.rotate_vertical) * self.sensitivity * dt * 0.4;

        // If process_mouse isn't called every frame, these values
        // will not get set to zero, and the camera will rotate
        // when moving in a non-cardinal direction.
        self.rotate_horizontal = 0.0;
        self.rotate_vertical = 0.0;

        iso.translation.x = cam_pos.x;
        iso.translation.y = cam_pos.y;
        iso.translation.z = cam_pos.z;

        iso.rotation = UnitQuaternion::from_euler_angles(0., pitch, yaw);

        *camera_rot_text = format!(
            "camera rotation: yaw: {:.2}°, pitch: {:.2}°",
            yaw.to_degrees(),
            pitch.to_degrees()
        );

        camera.forward = forward;
        camera.up = Vector::y();
        camera.right = right;
        camera.rot_matrix = rotation_matrix;
        camera.quat = rotation_quat;
    }
}
