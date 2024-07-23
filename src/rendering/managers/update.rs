use std::{
    any::TypeId,
    time::{Duration, Instant},
};

use crate::{
    engine_runtime::{
        schedule_manager::runtime_schedule::{
            RuntimeEndFrameData, RuntimeEndFrameSchedule, RuntimeUpdateSchedule,
        },
        EngineRuntime,
    },
    rendering::buffer_manager::dupe,
    typed_addr::TypedAddr,
    ENGINE_RUNTIME,
};

impl<'w> EngineRuntime<'w> {
    pub fn update(&mut self, dt: Duration, start: Instant) {
        let _dt_f64 = dt.as_secs_f64();
        let engine = unsafe { ENGINE_RUNTIME.get() };
        {
            let runtime_schedule_state: &mut RuntimeUpdateSchedule =
                dupe(engine).get_resource_mut();
            let update_state = runtime_schedule_state.schedule_state.get();

            update_state.dt = dt;
            update_state.since_start = Instant::now() - start;

            runtime_schedule_state.execute(dupe(engine));
        }
        /*{
            let schedule = engine.get_resource::<RuntimeEndFrameSchedule>();
            //let schedule_data = schedule.schedule_state.get();
            schedule.execute(dupe(engine));
        }*/
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
