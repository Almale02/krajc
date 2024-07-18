use crate::engine_runtime::schedule_manager::system_params::system_query::SystemQuery;
use crate::rendering::camera::camera::Camera;
use crate::typed_addr::dupe;
use crate::AspectUniform;
use crate::Mesh;
use crate::QueryFilter;
use crate::RenderManagerResource;
use crate::RuntimeUpdateScheduleData;
use crate::SchedData;

use krajc::system_fn;
use krajc::Comp;
use legion::internals::world::Comp;
use legion::query::ComponentFilter;
use legion::Read;
use legion::Write;
use rapier3d::math::Vector;
use rapier3d::na::Isometry3;

use crate::Res;

pub type Isometry = Isometry3<f32>;

#[derive(Default, Comp)]
struct Test {}

#[system_fn(RuntimeUpdateSchedule)]
pub fn update_rendering(
    camera: SystemQuery<Write<Isometry>, QueryFilter<Test>>,
    mut render_state: Res<RenderManagerResource>,
    update: SchedData<RuntimeUpdateScheduleData>,
) {
    let render_state = render_state.get_static_mut();

    let a = camera.query().get_single().unwrap();

    render_state
        .camera_controller
        .update_camera(&mut render_state.camera, update.dt.as_secs_f64());
    render_state
        .camera_uniform
        .update_view_proj(&render_state.camera, &render_state.projection);

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
