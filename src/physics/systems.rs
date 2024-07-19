use krajc::system_fn;
use rapier3d::dynamics::IntegrationParameters;

use crate::engine_runtime::schedule_manager::{
    runtime_schedule::RuntimeEndFrameSchedule,
    system_params::{system_query::Runtime, system_resource::Res},
};

use super::Gravity;

#[system_fn(RuntimeEndFrameSchedule)]
pub fn step_physics(runtime: Runtime, gravity: Res<Gravity>) {
    let world = runtime.physics;

    runtime.physics.physics_pipeline.step(
        &*gravity,
        &IntegrationParameters::default(),
        &mut world.island_manager,
        &mut world.broad_phase,
        world.narrow_phase,
        &mut world.rigid_body_set,
        &mut world.collider_set,
        &mut world.impulse_joint_set,
        &mut world.multibody_joint_set,
        &mut world.ccd_solver,
        Some(&mut world.query_pipeline),
        (),
        (),
    )
}
