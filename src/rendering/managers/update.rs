use std::{
    any::TypeId,
    time::{Duration, Instant},
};

use crate::{
    engine_runtime::{schedule_manager::runtime_schedule::RuntimeUpdateSchedule, EngineRuntime},
    typed_addr::TypedAddr,
    ENGINE_RUNTIME,
};

impl EngineRuntime {
    pub fn update(&mut self, dt: Duration, start: Instant) {
        let _dt_f64 = dt.as_secs_f64();
        {
            let engine = TypedAddr::new(
                *unsafe {
                    ENGINE_RUNTIME
                        .get()
                        .static_resource_map
                        .get(&TypeId::of::<RuntimeUpdateSchedule>())
                }
                .unwrap(),
            );

            let runtime_schedule_state: &mut RuntimeUpdateSchedule = engine.get();
            let update_state = runtime_schedule_state.schedule_state.get();

            *update_state.dt = dt;
            *update_state.since_start = Instant::now() - start;

            runtime_schedule_state.execute();
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
