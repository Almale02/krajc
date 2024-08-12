use std::ops::Deref;
use std::ops::DerefMut;

use crate::engine_runtime::schedule_manager::system_params::system_query::SystemQuery;
use crate::physics::components::general::LinearVelocity;
use crate::rendering::camera::camera::Camera;
use crate::RenderManagerResource;
use crate::RuntimeUpdateScheduleData;
use crate::TextureMaterialMarker;

use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use bevy_ecs::query::With;
//use cgmath::Vector3;
use krajc::system_fn;
//use krajc::system_fn;
use rapier3d::na::Isometry3;
use rapier3d::na::Vector3;

use crate::Res;

#[derive(Component, Clone, Default, PartialEq)]
pub struct Transform {
    pub iso: Isometry3<f32>,
}

impl Eq for Transform {}

impl Transform {
    pub fn new(iso: Isometry3<f32>) -> Self {
        Self { iso }
    }
    pub fn new_vec(vec: Vector3<f32>) -> Self {
        Self::new(Isometry3::new(vec, Default::default()))
    }
}

impl Deref for Transform {
    type Target = Isometry3<f32>;
    fn deref(&self) -> &Self::Target {
        &self.iso
    }
}
impl DerefMut for Transform {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.iso
    }
}

#[system_fn(RuntimeUpdateSchedule)]
pub fn update_rendering(
    mut camera: SystemQuery<&mut Transform, With<Camera>>, // you can see that i am using bevy for querying,
    mut render_state: Res<RenderManagerResource>,          // my own resource system
    update: Res<RuntimeUpdateScheduleData>, // data related to the schedule like delta time
) {
    let render_state = render_state.get_static_mut();

    let mut iso = match camera.get_single_mut() {
        Ok(x) => x,
        Err(e) => return,
    };
    //dbg!(&iso.iso);

    render_state
        .camera_controller
        .update_camera(&mut iso, update.dt.as_secs_f64());
    render_state
        .camera_uniform
        .update_view_proj(&mut iso, &render_state.projection);

    render_state
        .camera_buffer
        .set_data(*render_state.camera_uniform);
}

#[system_fn(RuntimeUpdateSchedule)]
pub fn move_stuff_up(
    mut objects: SystemQuery<(Entity, &mut Transform), With<TextureMaterialMarker>>,
) {
    objects.iter_mut().for_each(|(id, mut trans)| {
        trans.translation.y += 0.1 + (id.index() as f32) / 50000.;
    })
}

#[derive(Component)]
pub struct Light;
#[derive(Component)]
pub struct Color(pub wgpu::Color);

#[system_fn(RuntimeUpdateSchedule)]
pub fn sync_light(
    mut render_state: Res<RenderManagerResource>,
    mut light: SystemQuery<(&Transform, &Color), With<Light>>,
) {
    let (trans, color) = {
        if light.get_single().is_err() {
            return;
        } else {
            light.single()
        }
    };

    let new_pos = cgmath::Vector3::new(
        trans.translation.x,
        trans.translation.y,
        trans.translation.z,
    );

    render_state.light_uniform.position =
        cgmath::Vector4::new(new_pos.x, new_pos.y, new_pos.z, 0.).into();
    render_state.light_uniform.color = [color.0.r as f32, color.0.g as f32, color.0.b as f32, 0.];

    render_state
        .light_buffer
        .set_data(*render_state.light_uniform);
}
#[system_fn(RuntimeUpdateSchedule)]
pub fn make_light_follow_camera(
    mut camera: SystemQuery<&Transform, With<Camera>>,
    mut light: SystemQuery<(&mut Transform, &mut LinearVelocity), With<Light>>,
) {
    let (camera) = camera.single().iso;
    let (light, mut lin_vel) = match light.get_single_mut() {
        Ok(x) => x,
        Err(_) => return,
    };

    let light = light.iso;

    let dir = camera.translation.vector - light.translation.vector;

    lin_vel.0 = dir * 0.12;
}
