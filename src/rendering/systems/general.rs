use std::ops::Deref;
use std::ops::DerefMut;
use std::ops::Neg;

use crate::engine_runtime::schedule_manager::system_params::system_query::SystemQuery;
use crate::rendering::camera::camera::Camera;
use crate::AspectUniform;
use crate::Marker;
use crate::RenderManagerResource;
use crate::RuntimeUpdateScheduleData;
use crate::SchedData;

use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use bevy_ecs::query::With;
use bevy_ecs::query::Without;
use cgmath::Rotation3;
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
    mut camera: SystemQuery<&mut Transform, (With<Camera>, Without<Marker>)>, // you can see that i am using bevy for querying,
    mut render_state: Res<RenderManagerResource>, // my own resource system
    update: SchedData<RuntimeUpdateScheduleData>, // data related to the schedule like delta time
) {
    let render_state = render_state.get_static_mut();

    let mut iso = match camera.single_mut() {
        Ok(x) => x,
        Err(e) => return,
    };

    render_state
        .camera_controller
        .update_camera(&mut iso, update.dt.as_secs_f64());
    render_state
        .camera_uniform
        .update_view_proj(&mut iso, &render_state.projection);

    render_state
        .camera_buffer
        .set_data(*render_state.camera_uniform);

    let trans = iso;
    dbg!(trans.translation);
}

#[system_fn(RuntimeUpdateSchedule)]
pub fn move_stuff_up(mut objects: SystemQuery<(Entity, &mut Transform), With<Marker>>) {
    objects.iter_mut().for_each(|(id, mut trans)| {
        trans.translation.y += 0.1 + (id.index() as f32) / 50000.;
    })
}

#[system_fn(RuntimeUpdateSchedule)]
pub fn move_light(
    mut render_state: Res<RenderManagerResource>,
    mut camera: SystemQuery<&mut Transform, (With<Camera>, Without<Marker>)>,
) {
    let trans = camera.get_single();

    if trans.is_err() {
        return;
    }
    let trans = trans.unwrap();

    //let old_position: cgmath::Vector4<_> = render_state.light_uniform.position.into();
    let new_pos = cgmath::Vector3::new(
        trans.translation.x,
        trans.translation.y,
        trans.translation.z,
    );

    render_state.light_uniform.position =
        cgmath::Vector4::new(new_pos.x, new_pos.y, new_pos.z, 0.).into();

    render_state
        .light_buffer
        .set_data(*render_state.light_uniform);
}
