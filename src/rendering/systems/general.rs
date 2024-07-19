use std::ops::Deref;
use std::ops::DerefMut;

use crate::engine_runtime::schedule_manager::system_params::system_query::EcsWorld;
use crate::engine_runtime::schedule_manager::system_params::system_query::SystemQuery;
use crate::rendering::camera::camera::Camera;
use crate::typed_addr;
use crate::typed_addr::dupe;
use crate::AspectUniform;
use crate::Mesh;
use crate::QueryFilter;
use crate::RenderManagerResource;
use crate::RuntimeUpdateScheduleData;
use crate::SchedData;

use bevy_ecs::component::Component;
use bevy_ecs::query::With;
use bevy_ecs::system::Query;
use krajc::system_fn;
use krajc::Comp;
use rapier3d::math::Vector;
use rapier3d::na::Isometry3;

use crate::Res;

#[derive(Component)]
pub struct Isometry {
    _iso: Isometry3<f32>,
}

impl Deref for Isometry {
    type Target = Isometry3<f32>;
    fn deref(&self) -> &Self::Target {
        &self._iso
    }
}
impl DerefMut for Isometry {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self._iso
    }
}

#[system_fn(RuntimeUpdateSchedule)]
pub fn update_rendering(
    mut camera: SystemQuery<&mut Isometry, With<Camera>>,
    mut render_state: Res<RenderManagerResource>,
    update: SchedData<RuntimeUpdateScheduleData>,
    mut world: EcsWorld,
) {
    let render_state = render_state.get_static_mut();

    let mut iso = camera.single_mut();

    render_state
        .camera_controller
        .update_camera(&mut *iso, update.dt.as_secs_f64());
    render_state
        .camera_uniform
        .update_view_proj(&mut *iso, &render_state.projection);

    render_state
        .camera_buffer
        .set_data(*render_state.camera_uniform);

    render_state.queue.write_buffer(
        &render_state.camera_buffer_actual,
        0,
        bytemuck::cast_slice(&[*render_state.camera_uniform]),
    );

    let new_aspect_uniform = AspectUniform::from_size(*render_state.size);
    render_state.queue.write_buffer(
        &render_state.aspect_buffer,
        0,
        bytemuck::cast_slice(&[new_aspect_uniform]),
    );
}
