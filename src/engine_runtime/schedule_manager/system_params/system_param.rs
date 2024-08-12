use std::marker::PhantomData;

use mopa::mopafy;
use uuid::Uuid;

use crate::{
    engine_runtime::{
        schedule_manager::schedule::{IntoSystem, ScheduleRunnable},
        EngineRuntime,
    },
    implement_into_system,
    typed_addr::TypedAddr,
};

use super::system_resource::{EngineResource, Res};

pub struct SystemParam {
    pub engine: &'static mut EngineRuntime,
    pub position: u8,
    pub fn_name: &'static str,
}

pub trait IntoSystemParalellFilter {
    fn get_filterable(&self) -> Box<dyn SystemParalellFilter>;
}

mopa::mopafy!(SystemParalellFilter);
pub trait SystemParalellFilter: mopa::Any {
    /// this is used for checking if 2 systems can run paralell, in other words they are compatible with each other
    /// this returns true if two params are compatible, and false if they not, check the [SystemQueryFilterable][super::system_query::SystemQueryFilterable] as an example
    fn filter_against_param(&self, param: &Box<dyn SystemParalellFilter>) -> bool;
}

impl<T: EngineResource> From<SystemParam> for Res<T> {
    fn from(value: SystemParam) -> Self {
        let engine = value.engine;
        let mut new_self = Res::<T> {
            addr: TypedAddr::<_>::new(0),
        };
        new_self.find_addr(engine);

        new_self
    }
}

pub struct FunctionSystem<Func: 'static, Marker> {
    pub name: &'static str,
    pub function: Func,
    pub param_filters: Vec<Box<dyn SystemParalellFilter>>,
    _p: PhantomData<Marker>,
}

impl<Func: 'static, Marker> FunctionSystem<Func, Marker> {
    pub fn new(function: Func) -> Self {
        Self {
            name: Box::leak(Box::new(Uuid::new_v4().to_string())),
            function,
            param_filters: Vec::default(),
            _p: PhantomData,
        }
    }
}

macro_rules! impl_schedule_runnable {
    ($($param:ident),*) => {
        //impl<$($param),*, Func> ScheduleRunnable for (&'static str, Func, Vec<Box<dyn SystemParalellFilter>>, std::marker::PhantomData<fn($($param),*)>  )
        impl<$($param),*, Func> ScheduleRunnable for FunctionSystem<Func, fn($($param),*)>
        where
            $($param: From<SystemParam> + IntoSystemParalellFilter + 'static),*,
            Func: Fn($($param),*)
        {
            fn run(&mut self, runtime: &'static mut EngineRuntime) {
                let runtime = TypedAddr::<EngineRuntime>::new(runtime as *mut _ as usize);
                let mut position = 0;
                // Call the function
                (self.function)(
                    $(
                        std::convert::Into::<$param>::into(
                        {
                            position += 1;
                            let a = SystemParam {
                                engine: runtime.get(),
                                fn_name: self.name(),
                                position,
                            };
                            a
                        }),
                    )*
                );
            }
            fn setup_filter(&mut self, runtime: &'static mut EngineRuntime) {
                let runtime = TypedAddr::<EngineRuntime>::new(runtime as *mut _ as usize);
                let mut position = 0;
                    $(
                        position += 1;
                        let a = std::convert::Into::<$param>::into(SystemParam {
                            engine: runtime.get(),
                            fn_name: self.name(),
                            position,
                        });
                        self.param_filters.push(a.get_filterable());

                    )*

            }
            fn predicate(&self, _runtime: &'static EngineRuntime) -> bool {
                true
            }
            fn name(&self) -> &'static str {
                self.name
            }
            fn get_params_filters(&self) -> &Vec<Box<dyn SystemParalellFilter>> {
                &self.param_filters
            }
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

implement_into_system!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);
implement_into_system!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
implement_into_system!(A, B, C, D, E, F, G, H, I, J, K, L, M, N);
implement_into_system!(A, B, C, D, E, F, G, H, I, J, K, L, M);
implement_into_system!(A, B, C, D, E, F, G, H, I, J, K, L);
implement_into_system!(A, B, C, D, E, F, G, H, I, J, K);
implement_into_system!(A, B, C, D, E, F, G, H, I, J);
implement_into_system!(A, B, C, D, E, F, G, H, I);
implement_into_system!(A, B, C, D, E, F, G, H);
implement_into_system!(A, B, C, D, E, F, G);
implement_into_system!(A, B, C, D, E, F);
implement_into_system!(A, B, C, D, E);
implement_into_system!(A, B, C, D);
implement_into_system!(A, B, C);
implement_into_system!(A, B);
implement_into_system!(A);
