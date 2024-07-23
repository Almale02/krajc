use std::ops::{Deref, DerefMut};

use crate::typed_addr::TypedAddr;

use super::system_param::{IntoSystemParalellFilter, SystemParalellFilter, SystemParam};

struct SchedDataFilterable {}

impl SystemParalellFilter for SchedDataFilterable {
    fn filter_against_param(&self, param: &Box<(dyn SystemParalellFilter + 'static)>) -> bool {
        true
    }
}

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

/*impl<T> DerefMut for SchedData<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.addr.get()
    }
}*/
impl<'w, T> From<SystemParam<'w>> for SchedData<T> {
    fn from(value: SystemParam) -> Self {
        Self {
            addr: TypedAddr::<T>::new(value.schedule_data),
        }
    }
}
impl<T> IntoSystemParalellFilter for SchedData<T> {
    fn get_filterable(&self) -> Box<dyn SystemParalellFilter> {
        Box::new(SchedDataFilterable {})
    }
}
