use std::marker::PhantomData;

#[macro_export]
macro_rules! addr_ptr_to_ref_mut {
    ($ptr: expr, $ty: ty, $msg: expr) => {
        unsafe {
            if ($ptr as *mut $ty).is_null() {
                panic!(
                    "tried to convert a null mem addr to ref mut at the macro, msg was: {}",
                    $msg
                )
            }
            &mut *($ptr as *mut $ty)
        }
    };
    ($ptr: expr, $ty: ty, $msg: expr, $x: expr) => {
        unsafe { &mut *($ptr as *mut $ty) }
    };
}
#[macro_export]
macro_rules! addr_ptr_to_ptr {
    ($ptr: expr, $ty: ty) => {
        unsafe { (*$ptr as *mut $ty) }
    };
}

pub struct TypedAddr<T> {
    pub addr: usize,
    _p: PhantomData<T>,
}
impl<T> TypedAddr<T> {
    pub const fn new(addr: usize) -> TypedAddr<T> {
        TypedAddr::<T> {
            addr,
            _p: PhantomData,
        }
    }
    pub fn new_with_ref(addr: &mut T) -> TypedAddr<T> {
        TypedAddr::<T> {
            addr: addr as *mut T as usize,
            _p: PhantomData,
        }
    }
    #[allow(clippy::mut_from_ref)]
    pub fn get(&self) -> &'static mut T {
        addr_ptr_to_ref_mut!(
            self.addr,
            T,
            "called from TypedAddr get, get full information from rust backtrace in debug mode"
        )
    }
    pub const fn default() -> Self {
        Self::new(0)
    }
}
pub fn dupe<T>(value: &mut T) -> &'static mut T {
    unsafe { std::mem::transmute(value as *mut T) }
}
