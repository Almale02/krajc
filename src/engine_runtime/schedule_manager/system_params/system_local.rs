use std::ops::{Deref, DerefMut};

use crate::typed_addr::TypedAddr;

use super::system_param::{IntoSystemParalellFilter, SystemParalellFilter, SystemParam};

pub struct Local<T>
where
    T: 'static + Default,
{
    pub addr: TypedAddr<T>,
}
impl<T: Default> Deref for Local<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.addr.get()
    }
}

impl<T: Default> DerefMut for Local<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.addr.get()
    }
}
impl<'w, T: Default> From<SystemParam<'w>> for Local<T> {
    fn from(value: SystemParam<'w>) -> Self {
        let map = value.engine.system_locals.entry(value.fn_name).or_default();

        let any = map.entry(value.position).or_insert(Box::<T>::default());
        Self {
            addr: TypedAddr::new_with_ref(any.downcast_mut::<T>().unwrap()),
        }
    }
}

struct LocalFilterable {}
impl SystemParalellFilter for LocalFilterable {
    fn filter_against_param(&self, param: &Box<(dyn SystemParalellFilter + 'static)>) -> bool {
        true
    }
}

impl<T: Default> IntoSystemParalellFilter for Local<T> {
    fn get_filterable(&self) -> Box<dyn SystemParalellFilter> {
        Box::new(LocalFilterable {})
    }
}
