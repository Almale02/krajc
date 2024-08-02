/// creates the schedule that are necessary to run the engine and develop your game
use bytemuck::Contiguous;
use std::thread;

use crate::{
    create_schedule, create_schedule_main, generate_state_struct_non_resource,
    implement_schedule_main, span, typed_addr::dupe, ThreadRawPointer,
};
use std::{
    collections::{HashMap, HashSet},
    slice::IterMut,
    time::Duration,
    vec::Vec,
};

use super::{
    super::EngineRuntime, schedule::ScheduleRunnable,
    system_params::system_resource::EngineResource,
};
use crate::{implement_schedule, struct_with_default, typed_addr::TypedAddr};

pub type DepGraph = (
    Vec<(usize, std::collections::HashSet<usize>)>,
    HashMap<usize, &'static Box<dyn ScheduleRunnable>>,
);

create_schedule!(RuntimeEngineLoadSchedule, RuntimeEngineLoadData);

struct_with_default!(RuntimeUpdateSchedule {
    schedule_name: String = "update".into(),
    actions: Vec<Box<dyn ScheduleRunnable>> = Vec::default(),
    dep_graph: DepGraph = DepGraph::default(),
});
struct_with_default!(RuntimeUpdateScheduleData {
    dt: Duration = Duration::ZERO,
    since_start: Duration = Duration::ZERO,
});
implement_schedule!(RuntimeUpdateSchedule);

create_schedule!(RuntimePostUpdateSchedule, RuntimePostUpdateData);

create_schedule_main!(RuntimePhysicsSyncMainSchedule, RuntimePhysicsSyncMainData);

create_schedule!(RuntimePostPhysicsSyncSchedule, RuntimePostPhysicsSyncData);

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
