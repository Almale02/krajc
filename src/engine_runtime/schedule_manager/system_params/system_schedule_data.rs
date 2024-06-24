use std::ops::{Deref, DerefMut};

use crate::typed_addr::TypedAddr;

use super::system_param::SystemParam;

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
