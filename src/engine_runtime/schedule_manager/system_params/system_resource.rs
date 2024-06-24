use std::{
    ops::{Deref, DerefMut},
};

use crate::{
    engine_runtime::{
        EngineRuntime,
    },
    typed_addr::TypedAddr,
};

pub struct Res<T>
where
    T: 'static + EngineResource,
{
    pub addr: TypedAddr<T>,
}
impl<T: EngineResource> Res<T> {
    pub fn find_addr(&mut self, engine: &mut EngineRuntime) {
        let address = engine.get_resource::<T>();
        self.addr = TypedAddr::new_with_ref(address);
    }
}
impl<T: EngineResource> Deref for Res<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.addr.get()
    }
}

impl<T: EngineResource> DerefMut for Res<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.addr.get()
    }
}
impl<T: EngineResource> Res<T> {
    pub fn get_static(&self) -> &'static T {
        self.addr.get()
    }
}

impl<T: EngineResource> Res<T> {
    pub fn get_static_mut(&mut self) -> &'static mut T {
        self.addr.get()
    }
}
#[macro_export]
macro_rules! init_resource {
    ($res: ty) => {
        impl EngineResource for $res {
            fn init(engine: &mut EngineRuntime) -> &'static mut Self {
                let mgr = Box::new(Self::default());
                let leaked = Box::leak(mgr);
                let raw = leaked as *mut _;
                let schedule_state_addr = raw as usize;
                engine
                    .static_resource_map
                    .insert(std::any::TypeId::of::<$res>(), schedule_state_addr);

                leaked
            }
        }
    };
    ($res: ty, $typed_addr_static: expr) => {
        impl $crate::EngineResource for $res {
            fn init(engine: &mut EngineRuntime) -> &'static mut Self {
                let mgr = Box::new(Self::default());
                let leaked = Box::leak(mgr);
                let raw = leaked as *mut _;

                let schedule_state_addr = raw as usize;
                unsafe { $typed_addr_static.addr = schedule_state_addr }
                engine
                    .static_resource_map
                    .insert(std::any::TypeId::of::<$res>(), schedule_state_addr);

                leaked
            }
        }
    };
}
pub trait EngineResource {
    fn init(engine: &mut EngineRuntime) -> &'static mut Self;
}
