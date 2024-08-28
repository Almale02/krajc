use std::time::{Duration, Instant};

use rapier3d::dynamics::IntegrationParameters;

use crate::{
    drop_span,
    engine_runtime::{
        schedule_manager::{
            runtime_schedule::{
                RuntimePhysicsSyncMainSchedule, RuntimePostPhysicsSyncSchedule,
                RuntimePostUpdateSchedule, RuntimeUpdateSchedule, RuntimeUpdateScheduleData,
            },
            schedule::Schedule as _,
        },
        EngineRuntime,
    },
    physics::{
        physics_world::{
            BroadPhase, CcdSolver, ColliderSet, ImpulseJointSet, IslandManager, MultibodyJointSet,
            NarrowPhase, PhysicsPipeline, QueryPipeline, RigidBodySet,
        },
        Gravity,
    },
    typed_addr::dupe,
    ENGINE_RUNTIME,
};

impl EngineRuntime {
    pub fn update(&mut self, dt: Duration, start: Instant) {
        crate::span!(span, "update span");

        let _dt_f64 = dt.as_secs_f64();

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

        engine.get_resource_mut::<PhysicsPipeline>().0.step(
            &gravity.0,
            &IntegrationParameters::default(),
            &mut engine.get_resource_mut::<IslandManager>().0,
            &mut engine.get_resource_mut::<BroadPhase>().0,
            &mut engine.get_resource_mut::<NarrowPhase>().0,
            &mut engine.get_resource_mut::<RigidBodySet>().0,
            &mut engine.get_resource_mut::<ColliderSet>().0,
            &mut engine.get_resource_mut::<ImpulseJointSet>().0,
            &mut engine.get_resource_mut::<MultibodyJointSet>().0,
            &mut engine.get_resource_mut::<CcdSolver>().0,
            Some(&mut engine.get_resource_mut::<QueryPipeline>().0),
            &event_handler,
            &event_handler,
        );

        drop_span!(trace_physics);

        self.ecs.world.clear_trackers();
    }
}
