use bytemuck::Contiguous;
use cgmath::{num_traits::identities, Zero};
use krajc::EngineResource;
use std::{ops::DerefMut, thread};

use crate::{
    engine_runtime::{
        engine_data::data_refs::EngineDataRef, schedule_manager::schedule::calc_dep_graph,
    },
    generate_state_struct_non_resource,
    physics::Gravity,
    rendering::buffer_manager::{dupe, dupe_static},
    Lateinit, ThreadRawPointer, ENGINE_RUNTIME,
};
use std::{
    collections::{HashMap, HashSet},
    slice::IterMut,
    time::Duration,
    vec::Vec,
};

use super::{
    super::EngineRuntime,
    schedule::{
        single_thread_scheduler, MainToThreadExecutorMsg, ScheduleRunnable, ThreadExecutorToMainMsg,
    },
    system_params::{system_param::SystemParalellFilter, system_resource::EngineResource},
};
use crate::{implement_schedule, struct_with_default, typed_addr::TypedAddr};

pub type DepGraph<'w> = (
    EngineDataRef<'w, Vec<(usize, std::collections::HashSet<usize>)>>,
    HashMap<usize, &'w Box<dyn ScheduleRunnable>>,
);
/*pub static mut SCHEDULE_STATES: TypedAddr<RuntimeScheduleResource> = TypedAddr::<_>::default();

generate_state_struct!(RuntimeScheduleResource {
    engine_loaded: Schedule<RuntimeEngineLoadScheduleState> = "engine_loaded" => Schedule::new("engine_loaded", RuntimeEngineLoadScheduleState::init()),
    update: Schedule<RuntimeUpdateScheduleState> = "update" => Schedule::new("update", RuntimeUpdateScheduleState::init()),
}, SCHEDULE_STATES);*/

#[derive(EngineResource)]
pub struct RuntimeUpdateSchedule<'w>
where
    'w: 'static,
{
    pub schedule_name: String,
    pub actions: Vec<Box<dyn ScheduleRunnable>>,
    pub schedule_state: TypedAddr<RuntimeUpdateScheduleData>,
    pub dep_graph: Lateinit<DepGraph<'w>>,
}

impl<'w> Default for RuntimeUpdateSchedule<'w> {
    fn default() -> Self {
        Self {
            schedule_name: "update".into(),
            actions: Vec::default(),
            schedule_state: TypedAddr::new_with_ref(RuntimeUpdateScheduleData::init()),
            dep_graph: Default::default(),
        }
    }
}
generate_state_struct_non_resource!(RuntimeUpdateScheduleData {
    dt: Duration = Duration::ZERO,
    since_start: Duration = Duration::ZERO,
});

implement_schedule!(RuntimeUpdateSchedule);

#[derive(EngineResource)]
pub struct RuntimeEngineLoadSchedule<'w>
where
    'w: 'static,
{
    pub schedule_name: String,
    pub actions: Vec<Box<dyn ScheduleRunnable>>,
    pub schedule_state: TypedAddr<RuntimeEngineLoadScheduleData>,
    pub dep_graph: Lateinit<DepGraph<'w>>,
}
impl<'w> Default for RuntimeEngineLoadSchedule<'w> {
    fn default() -> Self {
        Self {
            schedule_name: "engine_load".into(),
            actions: Vec::default(),
            schedule_state: TypedAddr::new_with_ref(RuntimeEngineLoadScheduleData::init()),
            dep_graph: Default::default(),
        }
    }
}

generate_state_struct_non_resource!(RuntimeEngineLoadScheduleData { dummy: u32 = 0 });

implement_schedule!(RuntimeEngineLoadSchedule);

#[derive(EngineResource)]
pub struct RuntimeEndFrameSchedule<'w>
where
    'w: 'static,
{
    pub schedule_name: String,
    pub actions: Vec<Box<dyn ScheduleRunnable>>,
    pub schedule_state: TypedAddr<RuntimeEndFrameData>,
    pub dep_graph: Lateinit<DepGraph<'w>>,
}
impl<'w> Default for RuntimeEndFrameSchedule<'w> {
    fn default() -> Self {
        Self {
            schedule_name: "end_frame".into(),
            actions: Vec::default(),
            schedule_state: TypedAddr::new_with_ref(RuntimeEndFrameData::init()),
            dep_graph: Default::default(),
        }
    }
}
generate_state_struct_non_resource!(RuntimeEndFrameData { dummy: u32 = 0 });
implement_schedule!(RuntimeEndFrameSchedule);

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

/*impl<'a> RuntimeUpdateSchedule<'a> {
    pub fn new(name: &str, schedule_state_addr: usize) -> Self {
        Self {
            schedule_name: name.to_string(),
            actions: Vec::default(),
            schedule_state: TypedAddr::new(schedule_state_addr),
            dep_graph: Lateinit::default(),
        }
    }
    pub fn calc_dep_graph<'w: 'a>(&'w mut self, engine: &'w mut EngineRuntime<'w>)
    where
        'a: 'w,
    {
        let start = std::time::Instant::now();
        self.dep_graph
            .set(calc_dep_graph(&mut self.actions, engine));
        dbg!(start.elapsed());
    }
    pub fn execute(&'static mut self, engine: &'static mut EngineRuntime) {
        //dbg!(engine.paralellism);
        if !engine.paralellism {
            single_thread_scheduler(engine, &mut self.actions, self.schedule_state.addr);

            return;
        }

        let (main_tx, thread_rx) = flume::unbounded();
        let (thread_tx, main_rx) = flume::unbounded();

        let mut thread_join = vec![];

        let (dep_graph, ids) = &mut self.dep_graph.get(); //calc_dep_graph(&mut self.actions, dupe(engine));
                                                          //dbg!(dep_graph.clone());

        let mut to_execute = HashSet::new();
        let mut executed = HashSet::new();
        let mut active_deps: HashSet<usize> = ids.keys().copied().collect();

        let thread_num = thread::available_parallelism().unwrap().into_integer() - 1;

        for _i in 0..thread_num {
            let engine = TypedAddr::new_with_ref(engine);
            let schedule_address = self.schedule_state.addr;
            let tx = thread_tx.clone();
            let rx = thread_rx.clone();
            thread_join.push(thread::spawn(move || loop {
                match rx.try_recv() {
                    Ok(msg) => match msg {
                        MainToThreadExecutorMsg::Kill => {
                            drop(rx);
                            drop(tx);
                            return;
                        }
                        MainToThreadExecutorMsg::RunSystem(id, system) => {
                            let system = dupe(*system);
                            system.run(engine.get(), schedule_address);

                            tx.send(ThreadExecutorToMainMsg::SystemExecuted(id))
                                .unwrap();
                        }
                    },
                    Err(e) => match e {
                        flume::TryRecvError::Empty => continue,
                        flume::TryRecvError::Disconnected => return,
                    },
                }
            }));
        }

        for (id, deps) in dep_graph.iter() {
            if deps.is_disjoint(&active_deps) {
                to_execute.insert(id);
            }
        }
        let mut shall_run = true;
        loop {
            if !shall_run {
                break;
            }
            match main_rx.try_recv() {
                Ok(msg) => match msg {
                    ThreadExecutorToMainMsg::SystemExecuted(id) => {
                        active_deps.remove(&id);
                        executed.insert(id);

                        if executed.len() == ids.len() {
                            for _i in 0..thread_num {
                                main_tx.send(MainToThreadExecutorMsg::Kill).unwrap();
                            }
                            shall_run = false;
                            break;
                        }

                        for (id, deps) in dep_graph.iter() {
                            if executed.contains(id) {
                                continue;
                            }
                            if deps.is_disjoint(&active_deps) {
                                to_execute.insert(id);
                            }
                        }
                    }
                },
                Err(e) => match e {
                    flume::TryRecvError::Empty => (),
                    flume::TryRecvError::Disconnected => shall_run = false,
                },
            }

            let mut to_remove = Vec::new();
            for id in to_execute.iter() {
                main_tx
                    .send(MainToThreadExecutorMsg::RunSystem(
                        **id,
                        ThreadRawPointer::new(ids.get(&id).unwrap()),
                    ))
                    .unwrap();
                to_remove.push(*id);
            }
            for id in to_remove {
                to_execute.remove(id);
            }
        }
        for join in thread_join {
            let _ = join.join();
        }
        //dbg!(pre.elapsed());
    }
    pub fn register(&mut self, action: Box<dyn ScheduleRunnable>) {
        self.actions.push(action);
    }
}
unsafe impl<'w> Send for RuntimeUpdateSchedule<'w> {}
unsafe impl<'w> Sync for RuntimeUpdateSchedule<'w> {}*/
