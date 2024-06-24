use crate::{generate_state_struct_non_resource, ENGINE_RUNTIME};
use std::time::Duration;

use super::{
    super::EngineRuntime, schedule::ScheduleRunnable,
    system_params::system_resource::EngineResource,
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

generate_state_struct!(RuntimeEngineLoadScheduleData {
    dummy: u32 = "dummy" => 0
});
struct_with_default!(RuntimeEngineLoadSchedule{
    schedule_name: String = "engine_load".into(),
    actions: Vec<Box<dyn ScheduleRunnable>> = Vec::default(),
    schedule_state: TypedAddr<RuntimeEngineLoadScheduleData> = TypedAddr::default(),
});
implement_schedule!(RuntimeEngineLoadSchedule);
init_resource!(RuntimeEngineLoadSchedule);

fn a() {}
