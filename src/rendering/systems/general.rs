use crate::AspectUniform;
use crate::Mesh;
use crate::RenderManagerResource;
use crate::RuntimeUpdateScheduleData;
use crate::SchedData;

use krajc::system_fn;

use crate::Res;

#[system_fn(RuntimeUpdateSchedule)]
pub fn update_rendering(
    mut render_state: Res<RenderManagerResource>,
    update: SchedData<RuntimeUpdateScheduleData>,
) {
    let render_state = render_state.get_static_mut();

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
