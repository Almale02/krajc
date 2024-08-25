use std::{
    any::TypeId,
    ops::{Deref, DerefMut},
};

use crate::{engine_runtime::EngineRuntime, typed_addr::TypedAddr};

use super::system_param::{IntoSystemParalellFilter, SystemParalellFilter};

impl<T: 'static + EngineResource> IntoSystemParalellFilter for Res<T> {
    fn get_filterable(&self) -> Box<dyn SystemParalellFilter> {
        Box::new(ResFilterable(TypeId::of::<T>()))
    }
}

pub struct ResFilterable(TypeId);

impl SystemParalellFilter for ResFilterable {
    fn filter_against_param(&self, param: &Box<dyn SystemParalellFilter + 'static>) -> bool {
        match param.downcast_ref::<ResFilterable>() {
            Some(other) => other.0 != self.0,
            None => true,
        }
    }
}

pub struct Res<T>
where
    T: 'static + EngineResource,
{
    pub addr: TypedAddr<T>,
}
impl<T: EngineResource> Res<T> {
    pub fn find_addr(&mut self, engine: &'static mut EngineRuntime) {
        let address = engine.get_resource_mut::<T>();
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
pub trait EngineResource {
    fn get_mut(engine: &'static mut EngineRuntime) -> &'static mut Self;
    fn get(engine: &'static mut EngineRuntime) -> &'static Self;
    fn get_no_init(engine: &'static EngineRuntime) -> &'static Self;
}
