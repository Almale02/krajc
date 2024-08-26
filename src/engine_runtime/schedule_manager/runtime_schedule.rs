/// creates the schedule that are necessary to run the engine and develop your game
use bytemuck::Contiguous;
use std::thread;

use crate::{
    create_schedule, create_schedule_main, implement_schedule_main, typed_addr::dupe,
    ThreadRawPointer,
};
use std::{
    collections::{HashMap, HashSet},
    slice::IterMut,
    time::Duration,
    vec::Vec,
};

use super::schedule::ScheduleRunnable;
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
#[allow(unused_imports)]
use crate::engine_runtime::schedule_manager::schedule::*;
impl crate::engine_runtime::schedule_manager::schedule::Schedule for RuntimeUpdateSchedule {
    fn register_dyn(&mut self, action: Box<dyn ScheduleRunnable>) {
        self.actions.push(action);
    }
    fn register(&mut self, action: impl ScheduleRunnable + 'static) {
        println!("pushed registered: {}", action.name());
        self.register_dyn(Box::new(action) as Box<dyn ScheduleRunnable>);
    }
    fn execute(&'static mut self, engine: &'static mut crate::EngineRuntime) {
        crate::span!(trace_exec, stringify!(RuntimeUpdateSchedule));
        if !engine.paralellism {
            {
                let actions: &mut Vec<Box<dyn ScheduleRunnable>> = &mut self.actions;
                for action in actions.iter_mut() {
                    println!("ran system with name: {}", action.name());
                    action.run(dupe(engine));
                }
                //dbg!(thread_num);
            };
            return;
        }
        let (main_tx, thread_rx) = flume::unbounded();
        let (thread_tx, main_rx) = flume::unbounded();
        let mut thread_join = vec![];
        let (dep_graph, ids) = &mut self.dep_graph;
        let mut to_execute = HashSet::new();
        let mut executed = HashSet::new();
        let mut active_deps: HashSet<usize> = ids.keys().copied().collect();
        let thread_num = thread::available_parallelism().unwrap().into_integer() - 1;
        crate::span!(trace_start_exec, "paralell_exec");
        crate::span!(trace_thread_create, "create_threads");
        for _i in 0..thread_num {
            let engine = TypedAddr::new_with_ref(engine);
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
                            system.run(engine.get());
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
        crate::drop_span!(trace_thread_create);
        crate::span!(trace_start_main_thread, "thread_execution_main_loop");
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
        crate::span!(trace_join_threads, "join_threads");
        for join in thread_join {
            let _ = join.join();
        }
        crate::drop_span!(trace_join_threads);
        crate::drop_span!(trace_start_main_thread);
        crate::drop_span!(trace_start_exec);
    }
}
impl RuntimeUpdateSchedule {
    pub fn new(name: &str) -> Self {
        Self {
            schedule_name: name.to_string(),
            actions: Vec::default(),
            dep_graph: DepGraph::default(),
        }
    }
    pub fn calc_dep_graph(&'static mut self, engine: &mut crate::EngineRuntime) {
        let start = std::time::Instant::now();
        self.dep_graph = calc_dep_graph(&mut self.actions, dupe(engine));
        dbg!(start.elapsed());
    }
}
unsafe impl Send for RuntimeUpdateSchedule {}

unsafe impl Sync for RuntimeUpdateSchedule {}

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
