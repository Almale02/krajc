use crate::{engine_runtime::EngineRuntime, typed_addr::TypedAddr, ENGINE_RUNTIME};

pub struct Schedule<STATE> {
    pub schedule_name: String,
    pub actions: Vec<Box<dyn ScheduleRunnable>>,
    pub schedule_state: TypedAddr<STATE>,
}
impl<STATE> Schedule<STATE> {
    pub fn new(name: &str, schedule_state_addr: usize) -> Self {
        Self {
            schedule_name: name.to_string(),
            actions: Vec::default(),
            schedule_state: TypedAddr::new(schedule_state_addr),
        }
    }
    pub fn execute(&mut self) {
        unsafe {
            let runtime_raw = TypedAddr::new(ENGINE_RUNTIME.get() as *mut _ as usize);
            for action in &mut self.actions {
                if action.predicate(runtime_raw.get(), self.schedule_state.addr) {
                    action.run(runtime_raw.get(), self.schedule_state.addr)
                }
            }
        }
    }
    pub fn register(&mut self, action: Box<dyn ScheduleRunnable>) {
        self.actions.push(action);
    }
}
pub trait ScheduleRunnable {
    fn run(&mut self, runtime: &'static mut EngineRuntime, schedule_state: usize);
    fn predicate(&self, runtime: &'static EngineRuntime, schedule_state: usize) -> bool;
    fn name(&self) -> &'static str;
    fn setup_filter(&mut self, runtime: &'static mut EngineRuntime, schedule_state: usize);
}

#[macro_export]
macro_rules! implement_schedule {
    ($type: ty) => {
        impl $type {
            pub fn new(name: &str, schedule_state_addr: usize) -> Self {
                Self {
                    schedule_name: name.to_string(),
                    actions: Vec::default(),
                    schedule_state: TypedAddr::new(schedule_state_addr),
                }
            }
            pub fn execute(&mut self) {
                unsafe {
                    let runtime_raw = TypedAddr::new(ENGINE_RUNTIME.get() as *mut _ as usize);
                    for action in &mut self.actions {
                        if action.predicate(runtime_raw.get(), self.schedule_state.addr) {
                            action.run(runtime_raw.get(), self.schedule_state.addr)
                        }
                    }
                }
            }
            pub fn register(&mut self, action: Box<dyn ScheduleRunnable>) {
                self.actions.push(action);
            }
        }
    };
}
