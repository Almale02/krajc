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
    typed_addr::{dupe, TypedAddr},
    ENGINE_RUNTIME,
};

impl EngineRuntime {
    pub fn update(&mut self, dt: Duration, start: Instant) {
        let _dt_f64 = dt.as_secs_f64();
        let engine = unsafe { ENGINE_RUNTIME.get() };
        {
            let runtime_schedule_state: &mut RuntimeUpdateSchedule = engine.get_resource();
            let update_state = runtime_schedule_state.schedule_state.get();

            *update_state.dt = dt;
            *update_state.since_start = Instant::now() - start;

            runtime_schedule_state.execute(dupe(engine));
        }
        {
            let schedule = engine.get_resource::<RuntimeEndFrameSchedule>();
            let schedule_data = schedule.schedule_state.get();
        }
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
