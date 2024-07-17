use bytemuck::Contiguous;
use cgmath::{num_traits::identities, Zero};
use std::{ops::DerefMut, thread};

use crate::{
    generate_state_struct_non_resource, typed_addr::dupe, ThreadRawPointer, ENGINE_RUNTIME,
};
use std::{
    collections::{HashMap, HashSet},
    slice::IterMut,
    time::Duration,
    vec::Vec,
};

use super::{
    super::EngineRuntime,
    schedule::ScheduleRunnable,
    system_params::{system_param::SystemParalellFilter, system_resource::EngineResource},
};
use crate::{
    engine_runtime::engine_state_manager::generic_state_manager::GenericStateRefTemplate,
    implement_schedule, init_resource, struct_with_default, typed_addr::TypedAddr,
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
implement_schedule!(RuntimeEngineLoadSchedule);
init_resource!(RuntimeEngineLoadSchedule);

pub trait IterExt {
    type T;
    fn iter_mut_totallysafe(&self) -> IterMut<'_, Self::T>;
}

impl<T: 'static> IterExt for [T] {
    type T = T;
    fn iter_mut_totallysafe(&self) -> IterMut<'_, Self::T> {
        dupe(self).iter_mut()
    }
}
