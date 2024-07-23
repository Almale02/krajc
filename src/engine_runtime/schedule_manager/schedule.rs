use std::{
    collections::{HashMap, HashSet},
    thread,
};

use bytemuck::Contiguous;

use crate::{
    engine_runtime::{
        engine_data::data_refs::{EngineDataMut, EngineDataRef},
        schedule_manager::runtime_schedule::IterExt,
        EngineRuntime,
    },
    rendering::buffer_manager::dupe,
    typed_addr::TypedAddr,
    ThreadRawPointer, ENGINE_RUNTIME,
};

use super::{
    runtime_schedule::RuntimeEngineLoadSchedule, system_params::system_param::SystemParalellFilter,
};

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
    fn run<'w>(&mut self, runtime: &'w mut EngineRuntime<'w>, schedule_state: usize);
    fn predicate<'w>(&self, runtime: &'w EngineRuntime, schedule_state: usize) -> bool;
    fn name(&self) -> &'static str;
    fn setup_filter<'w>(&mut self, runtime: &'w mut EngineRuntime<'w>, schedule_state: usize);
    fn get_params_filters(&self) -> &Vec<Box<dyn SystemParalellFilter>>;
}

pub fn single_thread_scheduler(
    engine: &'static mut EngineRuntime,
    actions: &mut Vec<Box<dyn ScheduleRunnable>>,
    state: usize,
) {
    for action in actions.iter_mut() {
        action.run(dupe(engine), state);
    }
    //let a = calc_dep_graph(&mut self.actions, dupe(engine));
    let thread_num = thread::available_parallelism().unwrap().into_integer() - 1;
    dbg!(thread_num);
}
#[macro_export]
macro_rules! implement_schedule {
    ($type: ident) => {
        impl<'a> $type<'a> {
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
    };
}

pub fn calc_dep_graph<'w>(
    systems: &'w mut Vec<Box<dyn ScheduleRunnable>>,
    engine: &'w mut EngineRuntime<'w>,
) -> (
    EngineDataRef<'w, Vec<(usize, std::collections::HashSet<usize>)>>,
    HashMap<usize, &'w Box<dyn ScheduleRunnable>>,
) {
    /*let mut paralell_systems = ScheduleParalellizationData::default();
    for (i, action) in &mut dupe(systems).iter().enumerate() {
        paralell_systems.add_new(i, action);
    }*/

    let mut ids: HashMap<usize, &Box<dyn ScheduleRunnable>> = HashMap::default();

    let mut groups: Vec<HashSet<usize>> = Vec::default();

    for (i, system) in dupe(systems).iter_mut().enumerate() {
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

    let dep_graph_data = dupe(engine).engine_data.create_new(dep_graph);
    (dep_graph_data, ids)
}
fn check_if_compatible(
    first: &Vec<Box<dyn SystemParalellFilter>>,
    second: &Vec<Box<dyn SystemParalellFilter>>,
) -> bool {
    for param in first {
        for other_param in second {
            if !param.filter_against_param(other_param) {
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
