use std::{
    collections::{HashMap, HashSet},
    thread,
};

use bytemuck::Contiguous;

use crate::{
    engine_runtime::{schedule_manager::runtime_schedule::IterExt, EngineRuntime},
    typed_addr::{dupe, TypedAddr},
    ThreadRawPointer, ENGINE_RUNTIME,
};

use super::system_params::system_param::SystemParalellFilter;

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
    fn get_params_filters(&self) -> &Vec<Box<dyn SystemParalellFilter>>;
}

pub fn single_thread_scheduler(
    engine: &'static mut EngineRuntime,
    actions: &mut Vec<Box<dyn ScheduleRunnable>>,
    state: usize,
) {
    for action in actions.iter_mut() {
        crate::span!(trace, stringify!(action.name()));
        action.run(dupe(engine), state);
    }
    let thread_num = thread::available_parallelism().unwrap().into_integer() - 1;
    //dbg!(thread_num);
}
#[macro_export]
macro_rules! implement_schedule {
    ($type: ty) => {
        #[allow(unused_imports)]
        use $crate::engine_runtime::schedule_manager::schedule::*;

        impl $type {
            pub fn new(name: &str, schedule_state_addr: usize) -> Self {
                Self {
                    schedule_name: name.to_string(),
                    actions: Vec::default(),
                    schedule_state: TypedAddr::new(schedule_state_addr),
                    dep_graph: DepGraph::default(),
                }
            }
            pub fn calc_dep_graph(&'static mut self, engine: &mut EngineRuntime) {
                let start = std::time::Instant::now();
                self.dep_graph = calc_dep_graph(&mut self.actions, dupe(engine));
                dbg!(start.elapsed());
            }

            #[allow(unused_assignments)]
            pub fn execute(&'static mut self, engine: &'static mut EngineRuntime) {
                $crate::span!(trace_exec, stringify!($type));
                if !engine.paralellism {
                    single_thread_scheduler(engine, &mut self.actions, self.schedule_state.addr);

                    return;
                }

                let (main_tx, thread_rx) = flume::unbounded();
                let (thread_tx, main_rx) = flume::unbounded();

                let mut thread_join = vec![];

                let (dep_graph, ids) = &mut self.dep_graph; //calc_dep_graph(&mut self.actions, dupe(engine));
                                                            //dbg!(dep_graph.clone());

                let mut to_execute = HashSet::new();
                let mut executed = HashSet::new();
                let mut active_deps: HashSet<usize> = ids.keys().copied().collect();

                let thread_num = thread::available_parallelism().unwrap().into_integer() - 1;

                $crate::span!(trace_start_exec, "paralell_exec");
                $crate::span!(trace_thread_create, "create_threads");

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
                $crate::drop_span!(trace_thread_create);

                $crate::span!(trace_start_main_thread, "thread_execution_main_loop");

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
                $crate::span!(trace_join_threads, "join_threads");
                for join in thread_join {
                    let _ = join.join();
                }
                $crate::drop_span!(trace_join_threads);
                $crate::drop_span!(trace_start_main_thread);
                $crate::drop_span!(trace_start_exec);
            }
            pub fn register(&mut self, action: Box<dyn ScheduleRunnable>) {
                self.actions.push(action);
            }
        }
        unsafe impl Send for $type {}
        unsafe impl Sync for $type {}
    };
}

#[macro_export]
macro_rules! span {
    ($name: expr) => {
        tracing_tracy::client::span!($name)
    };
    ($var: ident, $name: expr) => {
        #[cfg(not(feature = "prod"))]
        let $var = $crate::span!($name);
    };
}

#[macro_export]
macro_rules! drop_span {
    ($var: ident) => {
        #[cfg(not(feature = "prod"))]
        drop($var);
    };
}

#[macro_export]
macro_rules! implement_schedule_main {
    ($type: ty) => {
        impl $type {
            pub fn new(name: &str, schedule_state_addr: usize) -> Self {
                Self {
                    schedule_name: name.to_string(),
                    actions: Vec::default(),
                    schedule_state: TypedAddr::new(schedule_state_addr),
                }
            }
            pub fn execute(&'static mut self, engine: &'static mut EngineRuntime) {
                //dbg!(engine.paralellism);
                single_thread_scheduler(engine, &mut self.actions, self.schedule_state.addr);
            }
            pub fn register(&mut self, action: Box<dyn ScheduleRunnable>) {
                self.actions.push(action);
            }
        }
    };
}
pub fn calc_dep_graph(
    systems: &'static mut [Box<dyn ScheduleRunnable>],
    engine: &'static mut EngineRuntime,
) -> (
    Vec<(usize, std::collections::HashSet<usize>)>,
    HashMap<usize, &'static Box<dyn ScheduleRunnable>>,
) {
    /*let mut paralell_systems = ScheduleParalellizationData::default();
    for (i, action) in &mut dupe(systems).iter().enumerate() {
        paralell_systems.add_new(i, action);
    }*/

    let mut ids: HashMap<usize, &Box<dyn ScheduleRunnable>> = HashMap::default();

    let mut groups: Vec<HashSet<usize>> = Vec::default();

    for (i, system) in systems.iter_mut_totallysafe().enumerate() {
        system.setup_filter(dupe(engine), 0);
        ids.insert(i, system);
    }

    let dep_graph: Vec<(usize, HashSet<usize>)> =
        ids.keys().map(|i| (*i, HashSet::new())).collect::<Vec<_>>();

    for (i, system) in systems.iter().enumerate() {
        let mut found_group = false;
        for group in groups.iter_mut_totallysafe() {
            let mut compatible_here = true;
            for other_system_id in group.iter().collect::<Vec<_>>() {
                if !check_if_compatible(
                    system.get_params_filters(),
                    ids.get(other_system_id).unwrap().get_params_filters(),
                ) {
                    compatible_here = false;
                }
            }
            if compatible_here {
                group.insert(i);
                found_group = true;
                break;
            }
        }
        if !found_group {
            groups.push({
                let mut a = HashSet::new();
                a.insert(i);
                a
            });
        }
    }
    //dbg!(&groups);

    for (i, deps) in dep_graph.iter_mut_totallysafe() {
        for (j, other_deps) in dep_graph.iter_mut_totallysafe() {
            if i == j {
                continue;
            }
            if !check_if_compatible(
                ids.get(i).unwrap().get_params_filters(),
                ids.get(j).unwrap().get_params_filters(),
            ) {
                deps.insert(*j);
                other_deps.insert(*i);
            }
        }
    }
    for (i, deps) in dep_graph.iter_mut_totallysafe() {
        for (j, other_deps) in dep_graph.iter_mut_totallysafe() {
            if *i == *j {
                continue;
            }

            for group in groups.iter() {
                if group.contains(i) {
                    dupe(deps).remove(j);
                    break;
                }
                if group.contains(j) {
                    dupe(other_deps).remove(i);
                    break;
                }
            }
        }
    }
    (dep_graph, ids)
}
fn check_if_compatible(
    first: &Vec<Box<dyn SystemParalellFilter>>,
    second: &Vec<Box<dyn SystemParalellFilter>>,
) -> bool {
    for param in first {
        for other_param in second {
            if !(param.filter_against_param(other_param) && other_param.filter_against_param(param))
            {
                return false;
            }
        }
    }
    true
}

pub enum MainToThreadExecutorMsg {
    Kill,
    RunSystem(usize, ThreadRawPointer<&'static Box<dyn ScheduleRunnable>>),
}
pub enum ThreadExecutorToMainMsg {
    SystemExecuted(usize),
}

#[macro_export]
macro_rules! create_schedule {
    ($sched_type: ident, $data_type: ident) => {
        struct_with_default!($sched_type {
            schedule_name: String = "update".into(),
            actions: Vec<Box<dyn ScheduleRunnable>> = Vec::default(),
            schedule_state: TypedAddr<$data_type> = TypedAddr::new_with_ref($data_type::init()),
            dep_graph: DepGraph = DepGraph::default(),
        });
        generate_state_struct_non_resource!($data_type {
            dummy: u32 = "dummy" => 0
        });
        implement_schedule!($sched_type);
    };
}
#[macro_export]
macro_rules! create_schedule_main {
    ($sched_type: ident, $data_type: ident) => {
        struct_with_default!($sched_type {
            schedule_name: String = "update".into(),
            actions: Vec<Box<dyn ScheduleRunnable>> = Vec::default(),
            schedule_state: TypedAddr<$data_type> = TypedAddr::new_with_ref($data_type::init()),
        });
        generate_state_struct_non_resource!($data_type {
            dummy: u32 = "dummy" => 0
        });
        implement_schedule_main!($sched_type);
    };
}
