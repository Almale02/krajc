use crate::{generate_state_struct_non_resource, ENGINE_RUNTIME};
use std::{
    thread::{self, Thread},
    time::Duration,
};

use super::{
    super::EngineRuntime,
    schedule::ScheduleRunnable,
    system_params::{system_param::SystemParalellFilter, system_resource::EngineResource},
};
use crate::{
    engine_runtime::engine_state_manager::generic_state_manager::GenericStateRefTemplate,
    generate_state_struct, implement_schedule, init_resource, struct_with_default,
    typed_addr::TypedAddr,
};

/*pub static mut SCHEDULE_STATES: TypedAddr<RuntimeScheduleResource> = TypedAddr::<_>::default();

generate_state_struct!(RuntimeScheduleResource {
    engine_loaded: Schedule<RuntimeEngineLoadScheduleState> = "engine_loaded" => Schedule::new("engine_loaded", RuntimeEngineLoadScheduleState::init()),
    update: Schedule<RuntimeUpdateScheduleState> = "update" => Schedule::new("update", RuntimeUpdateScheduleState::init()),
}, SCHEDULE_STATES);*/

generate_state_struct_non_resource!(RuntimeUpdateScheduleData {
    dt: Duration = "dt" => Duration::ZERO,
    since_start: Duration = "since_start" => Duration::ZERO,
});
struct_with_default!(RuntimeUpdateSchedule {
    schedule_name: String = "update".into(),
    actions: Vec<Box<dyn ScheduleRunnable>> = Vec::default(),
    schedule_state: TypedAddr<RuntimeUpdateScheduleData> = TypedAddr::new_with_ref(RuntimeUpdateScheduleData::init()),
});
implement_schedule!(RuntimeUpdateSchedule);
init_resource!(RuntimeUpdateSchedule);

generate_state_struct_non_resource!(RuntimeEngineLoadScheduleData {
    dummy: u32 = "dummy" => 0
});
struct_with_default!(RuntimeEngineLoadSchedule{
    schedule_name: String = "engine_load".into(),
    actions: Vec<Box<dyn ScheduleRunnable>> = Vec::default(),
    schedule_state: TypedAddr<RuntimeEngineLoadScheduleData> = TypedAddr::new_with_ref(RuntimeEngineLoadScheduleData::init())
});
//implement_schedule!(RuntimeEngineLoadSchedule);
init_resource!(RuntimeEngineLoadSchedule);

impl RuntimeEngineLoadSchedule {
    pub fn new(name: &str, schedule_state_addr: usize) -> Self {
        Self {
            schedule_name: name.to_string(),
            actions: Vec::default(),
            schedule_state: TypedAddr::new(schedule_state_addr),
        }
    }
    pub fn execute(&mut self) {
        unsafe {
            let runtime_raw: TypedAddr<EngineRuntime> =
                TypedAddr::new(ENGINE_RUNTIME.get() as *mut _ as usize);

            let compatible_actions: Vec<Vec<&mut Box<dyn ScheduleRunnable>>> = Vec::default();

            for action in &mut self.actions {
                let params: &Vec<Box<dyn SystemParalellFilter>> = action.get_params_filters();
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

fn check_if_compatible(
    first: &Vec<Box<dyn SystemParalellFilter>>,
    second: &Vec<Box<dyn SystemParalellFilter>>,
) -> bool {
    for param in first {
        for other_param in second {
            if param.filter_against_param(other_param) == false {
                return false;
            }
        }
    }
    return true;
}
