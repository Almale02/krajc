use crate::{
    engine_runtime::{schedule_manager::schedule::ScheduleRunnable, EngineRuntime},
    typed_addr::TypedAddr,
};

use super::system_resource::{EngineResource, Res};

pub struct SystemParam {
    pub engine: &'static mut EngineRuntime,
    pub schedule_data: usize,
    pub position: u8,
    pub fn_name: &'static str,
}

impl<T: EngineResource> From<SystemParam> for Res<T> {
    fn from(value: SystemParam) -> Self {
        let engine = value.engine;
        let mut new_self = Res::<T> {
            addr: TypedAddr::<_>::new(0),
        };
        new_self.find_addr(engine);
        return new_self;
    }
}

trait IntoSchedRun {
    fn as_sched_run_box(&'static self) -> Box<dyn ScheduleRunnable>;
}
macro_rules! impl_schedule_runnable {
    ($($param:ident),*) => {
        impl<$($param),*> ScheduleRunnable for (&'static str, Box<dyn Fn($($param),*)>)
        where
            $($param: From<SystemParam>),*
        {
            fn run(&mut self, runtime: &'static mut EngineRuntime, schedule_state: usize) {
                let runtime = TypedAddr::<EngineRuntime>::new(runtime as *mut _ as usize);
                let mut position = 0;
                // Call the function
                self.1(
                    $(
                        std::convert::Into::<$param>::into(
                        {
                            position += 1;
                            SystemParam {
                                engine: runtime.get(),
                                schedule_data: schedule_state,
                                fn_name: self.name(),
                                position,
                            }

                        }),
                    )*
                );
            }
            fn predicate(&self, _runtime: &'static EngineRuntime, _schedule_state: usize) -> bool {
                true
            }
            fn name(&self) -> &'static str {
                self.0
            }
        }
    };
}

#[macro_export]
macro_rules! create_system {
    ($sys_name: ident ($($param: ident : $param_type: ty),*) $block: block) => {
        pub fn $sys_name($($param: $param_type),*) {
            $block
        }
    };
}

impl_schedule_runnable!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);
impl_schedule_runnable!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
impl_schedule_runnable!(A, B, C, D, E, F, G, H, I, J, K, L, M, N);
impl_schedule_runnable!(A, B, C, D, E, F, G, H, I, J, K, L, M);
impl_schedule_runnable!(A, B, C, D, E, F, G, H, I, J, K, L);
impl_schedule_runnable!(A, B, C, D, E, F, G, H, I, J, K);
impl_schedule_runnable!(A, B, C, D, E, F, G, H, I, J);
impl_schedule_runnable!(A, B, C, D, E, F, G, H, I);
impl_schedule_runnable!(A, B, C, D, E, F, G, H);
impl_schedule_runnable!(A, B, C, D, E, F, G);
impl_schedule_runnable!(A, B, C, D, E, F);
impl_schedule_runnable!(A, B, C, D, E);
impl_schedule_runnable!(A, B, C, D);
impl_schedule_runnable!(A, B, C);
impl_schedule_runnable!(A, B);
impl_schedule_runnable!(A);
