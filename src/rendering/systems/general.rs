use std::ops::Deref;
use std::ops::DerefMut;

use crate::engine_runtime::schedule_manager::system_params::system_query::EcsWorld;
use crate::engine_runtime::schedule_manager::system_params::system_query::SystemQuery;
use crate::rendering::camera::camera::Camera;
use crate::typed_addr;
use crate::typed_addr::dupe;
use crate::AspectUniform;
use crate::Marker;
use crate::Mesh;
use crate::RenderManagerResource;
use crate::RuntimeUpdateScheduleData;
use crate::SchedData;

use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use bevy_ecs::query::With;
use bevy_ecs::query::Without;
use bevy_ecs::system::Query;
//use cgmath::Vector3;
use krajc::system_fn;
//use krajc::system_fn;
use krajc::Comp;
use rapier3d::math::Vector;
use rapier3d::na::Isometry3;
use rapier3d::na::Vector3;

use crate::Res;

#[derive(Component, Clone, Default, PartialEq)]
pub struct Transform {
    iso: Isometry3<f32>,
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

    let mut iso = camera.single_mut();

    render_state
        .camera_controller
        .update_camera(&mut iso, update.dt.as_secs_f64());
    render_state
        .camera_uniform
        .update_view_proj(&mut iso, &render_state.projection);

    render_state
        .camera_buffer
        .set_data(*render_state.camera_uniform);

    /*render_state.queue.write_buffer(
        &render_state.camera_buffer_actual,
        0,
        bytemuck::cast_slice(&[*render_state.camera_uniform]),
    );*/

    let new_aspect_uniform = AspectUniform::from_size(*render_state.size);
    render_state.queue.write_buffer(
        &render_state.aspect_buffer,
        0,
        bytemuck::cast_slice(&[new_aspect_uniform]),
    );
}

#[system_fn(RuntimeUpdateSchedule)]
pub fn move_stuff_up(mut objects: SystemQuery<(Entity, &mut Transform), With<Marker>>) {
    objects.iter_mut().for_each(|(id, mut trans)| {
        trans.translation.y += 0.001 + (id.index() as f32) / 50000.;
    })
}
