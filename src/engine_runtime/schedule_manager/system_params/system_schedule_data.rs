use std::ops::{Deref, DerefMut};

use crate::typed_addr::TypedAddr;

use super::system_param::{SystemParalellFilter, SystemParam};

pub struct SchedData<T: 'static> {
    addr: TypedAddr<T>,
}
impl<T> Deref for SchedData<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.addr.get()
    }
}

impl<T> DerefMut for SchedData<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.addr.get()
    }
}
impl<T> From<SystemParam> for SchedData<T> {
    fn from(value: SystemParam) -> Self {
        Self {
            addr: TypedAddr::<T>::new(value.schedule_data),
        }
    }
}
impl<T> SystemParalellFilter for SchedData<T> {
    fn filter_against_param(&self, param: Box<dyn std::any::Any>) -> bool {
        true
    }

    fn get_filterable(&self) -> Box<dyn std::any::Any> {
        Box::new(0)
    }
}
