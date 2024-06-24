use std::{ops::{Deref, DerefMut}};

use crate::{typed_addr::TypedAddr};

use super::{system_param::SystemParam};

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
impl<T: Default> From<SystemParam> for Local<T> {
    fn from(value: SystemParam) -> Self {
        let map = value.engine.system_locals.entry(value.fn_name).or_default();
        
        let any = map.entry(value.position).or_insert(Box::<T>::default());
        Self {addr: TypedAddr::new_with_ref(any.downcast_mut::<T>().unwrap())}
    }
}
