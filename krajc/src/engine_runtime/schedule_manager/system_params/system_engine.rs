use crate::engine_runtime::EngineRuntime;

use super::system_param::{IntoSystemParalellFilter, SystemParalellFilter, SystemParam};

pub struct UnsafeEngineAccess {
    pub engine: &'static mut EngineRuntime,
}

impl std::ops::DerefMut for UnsafeEngineAccess {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.engine
    }
}

impl std::ops::Deref for UnsafeEngineAccess {
    type Target = &'static mut EngineRuntime;

    fn deref(&self) -> &Self::Target {
        &self.engine
    }
}
impl From<SystemParam> for UnsafeEngineAccess {
    fn from(value: SystemParam) -> Self {
        Self {
            engine: value.engine,
        }
    }
}
impl IntoSystemParalellFilter for UnsafeEngineAccess {
    fn get_filterable(&self) -> Box<dyn super::system_param::SystemParalellFilter + 'static> {
        Box::new(NoNoNever)
    }
}
struct NoNoNever;
impl SystemParalellFilter for NoNoNever {
    fn filter_against_param(&self, param: &Box<dyn SystemParalellFilter + 'static>) -> bool {
        false
    }
}

pub fn dump_memory<T>(data: &T) {
    let size = std::mem::size_of::<T>();
    let data_ptr = data as *const T as *const u8;
    let data_slice = unsafe { std::slice::from_raw_parts(data_ptr, size) };

    for byte in data_slice {
        print!("{:02X} ", byte);
    }
    println!();
}
