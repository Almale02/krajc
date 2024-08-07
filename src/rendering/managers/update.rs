use std::time::{Duration, Instant};

use krajc::not_prod;
use log::{Level, LevelFilter};
use rapier3d::{dynamics::IntegrationParameters, utils::smallest_abs_diff_between_angles};
use tracing_tracy::client::{frame_mark, SpanLocation};

use crate::{
    drop_span,
    engine_runtime::{
        schedule_manager::runtime_schedule::{
            RuntimePhysicsSyncMainSchedule, RuntimePostPhysicsSyncSchedule,
            RuntimePostUpdateSchedule, RuntimeUpdateSchedule, RuntimeUpdateScheduleData,
        },
        EngineRuntime,
    },
    physics::{physics_world::PhysicsWorld, Gravity},
    typed_addr::dupe,
    ENGINE_RUNTIME,
};

impl EngineRuntime {
    pub fn update(&mut self, dt: Duration, start: Instant) {
        crate::span!(span, "update span");

        let _dt_f64 = dt.as_secs_f64();

        let physics = self.get_resource_mut::<PhysicsWorld>();
        let gravity = self.get_resource_mut::<Gravity>();

        let engine = unsafe { ENGINE_RUNTIME.get() };
        {
            let runtime_schedule_state: &mut RuntimeUpdateSchedule = engine.get_resource_mut();
            let update_state = engine.get_resource_mut::<RuntimeUpdateScheduleData>();

            update_state.dt = dt;
            update_state.since_start = Instant::now() - start;

            runtime_schedule_state.execute(dupe(engine));
        }
        {
            let schedule = engine.get_resource_mut::<RuntimePostUpdateSchedule>();
            //let schedule_data = schedule.schedule_state.get();
            schedule.execute(dupe(engine));
        }
        {
            let schedule = engine.get_resource_mut::<RuntimePhysicsSyncMainSchedule>();
            //let schedule_data = schedule.schedule_state.get();
            schedule.execute(dupe(engine));
        }
        {
            let schedule = engine.get_resource_mut::<RuntimePostPhysicsSyncSchedule>();
            //let schedule_data = schedule.schedule_state.get();
            schedule.execute(dupe(engine));
        }
        let event_handler = ();

        crate::span!(trace_physics, "physics");

        dupe(&physics).physics_pipeline.step(
            &gravity.0,
            &IntegrationParameters::default(),
            &mut dupe(&physics).island_manager,
            &mut dupe(&physics).broad_phase,
            &mut dupe(&physics).narrow_phase,
            &mut dupe(&physics).rigid_body_set,
            &mut dupe(&physics).collider_set,
            &mut dupe(&physics).impulse_joint_set,
            &mut dupe(&physics).multibody_joint_set,
            &mut dupe(&physics).ccd_solver,
            Some(&mut dupe(&physics).query_pipeline),
            &event_handler,
            &event_handler,
        );

        drop_span!(trace_physics);

        self.ecs.world.clear_trackers();
    }
}

trait ContextExt<T> {
    fn set(&mut self, x: impl FnOnce(&T) -> T);
    fn with(&self, x: impl FnOnce(&T));
    fn with_mut(&mut self, x: impl FnOnce(&mut T));
}
impl<T> ContextExt<T> for T {
    fn set(&mut self, x: impl FnOnce(&T) -> T) {
        *self = x(self)
    }
    fn with(&self, x: impl FnOnce(&T)) {
        x(self)
    }
    fn with_mut(&mut self, x: impl FnOnce(&mut T)) {
        x(self)
    }
}
