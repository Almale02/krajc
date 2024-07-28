use std::{
    borrow::BorrowMut,
    collections::HashMap,
    hash::Hash,
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use crate::{addr_ptr_to_ref_mut, ENGINE_RUNTIME};

use super::*;

type RefCount = u128;
type RefMutCount = u128;

#[derive(Clone)]
pub struct GenericStateRefTemplate<T: 'static> {
    state_manager: usize,
    name: &'static str,
    _p: PhantomData<T>,
}
impl<T> Default for GenericStateRefTemplate<T> {
    fn default() -> Self {
        Self {
            state_manager: 0,
            name: "",
            _p: PhantomData,
        }
    }
}
impl<T> GenericStateRefTemplate<T> {
    pub fn new(name: &'static str) -> Self {
        let runtime = unsafe { ENGINE_RUNTIME.get() };
        let new_self = Self {
            state_manager: (runtime.state.generic.borrow_mut() as *mut _) as usize,
            name,
            _p: PhantomData,
        };
        new_self
    }
    pub fn new_and_init(name: &'static str, value: T) -> Self {
        let runtime = unsafe { ENGINE_RUNTIME.get() };

        let new_self = Self {
            state_manager: (runtime.state.generic.borrow_mut() as *mut _) as usize,
            name,
            _p: PhantomData,
        };
        new_self.init(value);
        new_self
    }
    pub fn init(&self, value: T) {
        let state_manager = addr_ptr_to_ref_mut!(
            self.state_manager,
            GenericStateManager,
            "ran from generic state template init accessed GenericStateManager"
        );
        state_manager.init_value(self.name, value);
    }
    pub fn get_ref(&self) -> GenericStateRef<T> {
        addr_ptr_to_ref_mut!(
            self.state_manager,
            GenericStateManager,
            "ran from generic ref template get ref"
        )
        .get_ref(self.name)
    }
    pub fn get_ref_mut(&self) -> GenericStateRefMut<T> {
        addr_ptr_to_ref_mut!(
            self.state_manager,
            GenericStateManager,
            "ran from generic ref template get ref mut"
        )
        .get_ref_mut(self.name)
    }
}
impl<T> Deref for GenericStateRefTemplate<T> {
    type Target = T;
    fn deref(&self) -> &'static Self::Target {
        self.get_ref().unref()
    }
}
impl<T> DerefMut for GenericStateRefTemplate<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            let state_mgr = addr_ptr_to_ref_mut!(
                self.state_manager,
                GenericStateManager,
                "ran from generic ref template deref mut"
            );
            //println!("the name was: {} aaa", self.name);
            let mut id = 0;
            if let Some(id_) = state_mgr.named_keys.get(self.name) {
                id = *id_;
            } else {
                panic!("key {} wasnt found in states", self.name);
            }
            if !(*(*addr_ptr_to_ref_mut!(
                self.state_manager,
                GenericStateManager,
                "ran from generic ref template deref mut"
            ))
            .get_value_mut(id, "ran from deref mut"))
            .value
            .is::<T>()
            {
                panic!("generic state from deref mut was unitied");
            }

            (*(*addr_ptr_to_ref_mut!(
                self.state_manager,
                GenericStateManager,
                "ran from generic ref template deref mut"
            ))
            .get_value_mut(id, "ran from deref mut"))
            .value
            .downcast_mut()
            .unwrap()
        }
    }
}

pub struct GenericStateManager {
    pub data: HashMap<u128, GenericEngineStateManagerValue>,
    pub named_keys: HashMap<String, u128>,
    pub new_key: u128,
}
impl<'engine> Default for GenericStateManager {
    fn default() -> Self {
        Self::new()
    }
}

impl<'engine> GenericStateManager {
    pub fn new() -> Self {
        Self {
            data: HashMap::default(),
            named_keys: HashMap::default(),
            new_key: 0,
        }
    }
    fn init_value<T: 'static>(&mut self, name: &str, value: T) -> u128 {
        println!(
            "inited {} with value type of {} at id {}",
            name,
            std::any::type_name_of_val(&value),
            self.new_key
        );
        self.data.insert(
            self.new_key,
            GenericEngineStateManagerValue {
                value: Box::new(value),
                ref_count: 1,
                ref_mut_count: 1,
            },
        );
        let curr_new = self.new_key;
        self.named_keys.insert(name.to_string(), curr_new);
        self.new_key += 1;

        curr_new
    }
    fn create_ref_by_id<T>(&mut self, id: u128) -> GenericStateRef<T> {
        self.inc_ref(id, "ran inc from create_ref_by_id");
        GenericStateRef::<T> {
            id,
            render_resource_manager: self as *mut _,
            _p: PhantomData,
        }
    }
    fn create_ref_mut_by_id<T>(&mut self, id: u128) -> GenericStateRefMut<T> {
        self.inc_ref(id, "ran inc from create_ref_by_id");
        self.inc_ref_mut(id, "ran inc from create_ref_by_id");
        GenericStateRefMut::<T> {
            id,
            render_resource_manager: self as *mut _,
            _p: PhantomData,
        }
    }
    fn inc_ref(&mut self, id: u128, message: &str) {
        unsafe {
            self.get_value_mut(
                id,
                format!("ran from inc_ref, id was {}, message {}", id, message).as_str(),
            )
            .as_mut()
            .unwrap()
            .ref_count += 1;
        }
    }
    fn inc_ref_mut(&mut self, id: u128, message: &str) {
        unsafe {
            self.get_value_mut(
                id,
                format!("ran from inc_ref, id was {}, message {}", id, message).as_str(),
            )
            .as_mut()
            .unwrap()
            .ref_mut_count += 1;
        }
    }
    fn get_value(&self, id: u128) -> *const GenericEngineStateManagerValue {
        self.
            data
            .get(&id).expect("cannot find already created RenderResourceRef in RenderResourcesManager
it cannot happen because only way to create a RenderResourceRef is to add it to RenderResourceManager")
    }
    fn get_value_mut(&mut self, id: u128, message: &str) -> *mut GenericEngineStateManagerValue {
        /*if id != 0 {
            panic!("id wasnt 0");
        }*/
        let _a = self.data.get(&id);
        /*println!(
            "return was {:?} wran wrom get_value_mut, message was {}, id was {}, {:?}",
            a,
            message,
            id,
            self.data.keys().collect::<Vec<_>>()
        );*/
        self.data.get_mut(&id).unwrap_or_else(|| panic!("cannot find already created RenderResourceRef in RenderResourcesManager,
it cannot happen because only way to create a RenderResourceRef is to add it to RenderResourceManager, message is {}", message))
    }
    pub fn register_new<T>(&mut self, value: Box<dyn Any>, name: &str) -> GenericStateRef<T> {
        let new_id = self.new_key;
        self.named_keys.insert(name.to_string(), new_id);
        self.data
            .insert(new_id, GenericEngineStateManagerValue::new(value));

        let resource_ref = self.create_ref_by_id(self.new_key);
        self.new_key += 1;
        resource_ref
    }
    pub fn register_new_and_get_mut<T>(
        &mut self,
        value: Box<dyn Any>,
        name: &str,
    ) -> GenericStateRefMut<T> {
        let resource_ref = self.create_ref_mut_by_id(self.new_key);
        self.named_keys.insert(name.to_string(), resource_ref.id);
        self.data
            .insert(resource_ref.id, GenericEngineStateManagerValue::new(value));

        self.new_key += 1;
        resource_ref
    }
    pub fn get_ref<T>(&mut self, name: &str) -> GenericStateRef<T> {
        let id = self.named_keys.get(name);
        match id {
            Some(id) => self.create_ref_by_id::<T>(*id),
            None => panic!("couldnt find state in general with name: {}", name),
        }
    }
    pub fn get_ref_mut<T>(&mut self, name: &str) -> GenericStateRefMut<T> {
        let id = self.named_keys.get(name);
        match id {
            Some(id) => self.create_ref_mut_by_id::<T>(*id),
            None => panic!("couldnt find state in general with name: {}", name),
        }
    }
    pub fn get_ref_count(&self, id: u128) -> (RefCount, RefMutCount) {
        unsafe {
            let val = self.get_value(id).as_ref().unwrap();
            (val.ref_count, val.ref_mut_count)
        }
    }
}

pub struct GenericStateRef<T: 'static> {
    id: u128,
    render_resource_manager: *mut GenericStateManager,
    _p: PhantomData<T>,
}
impl<T> GenericStateRef<T> {
    pub fn ref_count(&self) -> (RefCount, RefMutCount) {
        unsafe { (*self.render_resource_manager).get_ref_count(self.id) }
    }
    pub fn unref(&self) -> &'static T {
        unsafe {
            /*dbg!(if (*(*self.render_resource_manager).get_value(self.id))
                .value
                .deref()
                .type_id()
                == std::any::TypeId::of::<T>()
            {
                format!("same {}", std::any::type_name::<T>())
            } else {
                panic!("in generic state ref the underlying data was not the same as the type of the reference, expected: {}", std::any::type_name::<T>());
            });*/
            (*(*self.render_resource_manager).get_value(self.id))
                .value
                .downcast_ref::<T>()
                .unwrap_or_else(|| panic!("generic state manager unwrapped on {}", self.id))
        }
    }
}
impl<T> Deref for GenericStateRef<T> {
    type Target = T;
    fn deref(&self) -> &'static Self::Target {
        unsafe {
            /*dbg!(if (*(*self.render_resource_manager).get_value(self.id))
                .value
                .deref()
                .type_id()
                == std::any::TypeId::of::<T>()
            {
                format!("same {}", std::any::type_name::<T>())
            } else {
                panic!("in generic state ref the underlying data was not the same as the type of the reference, expected: {}", std::any::type_name::<T>());
            });*/
            (*(*self.render_resource_manager).get_value(self.id))
                .value
                .downcast_ref::<T>()
                .unwrap()
        }
    }
}
impl<T> Clone for GenericStateRef<T> {
    fn clone(&self) -> Self {
        unsafe {
            (*self.render_resource_manager).inc_ref(self.id, "ran inc from clone");
            Self {
                id: self.id,
                render_resource_manager: self.render_resource_manager,
                _p: PhantomData,
            }
        }
    }
}

impl<T> PartialEq for GenericStateRef<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl<T> Eq for GenericStateRef<T> {}
impl<T> Drop for GenericStateRef<T> {
    fn drop(&mut self) {
        unsafe {
            (*self.render_resource_manager)
                .get_value_mut(self.id, "ran from drop")
                .as_mut()
                .unwrap()
                .ref_count -= 1;
        }
    }
}
impl<T> Hash for GenericStateRef<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state)
    }
}
pub struct GenericStateRefMut<T: 'static> {
    id: u128,
    render_resource_manager: *mut GenericStateManager,
    _p: PhantomData<T>,
}
impl<T> GenericStateRefMut<T> {
    pub fn ref_count(&self) -> (RefCount, RefMutCount) {
        unsafe { (*self.render_resource_manager).get_ref_count(self.id) }
    }
}
impl<T> Deref for GenericStateRefMut<T> {
    type Target = T;
    fn deref(&self) -> &'static Self::Target {
        unsafe {
            (*(*self.render_resource_manager).get_value(self.id))
                .value
                .downcast_ref()
                .unwrap()
        }
    }
}
impl<T> DerefMut for GenericStateRefMut<T> {
    fn deref_mut(&mut self) -> &'static mut Self::Target {
        unsafe {
            (*(*self.render_resource_manager)
                .get_value_mut(self.id, "ran from deref mut".to_string().as_str()))
            .value
            .downcast_mut()
            .unwrap()
        }
    }
}
impl<T> Clone for GenericStateRefMut<T> {
    fn clone(&self) -> Self {
        unsafe {
            (*self.render_resource_manager).inc_ref(self.id, "ran inc from clone");
            (*self.render_resource_manager).inc_ref_mut(self.id, "ran inc from clone");
            Self {
                id: self.id,
                render_resource_manager: self.render_resource_manager,
                _p: PhantomData,
            }
        }
    }
}

impl<T> PartialEq for GenericStateRefMut<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl<T> Eq for GenericStateRefMut<T> {}
impl<T> Drop for GenericStateRefMut<T> {
    fn drop(&mut self) {
        unsafe {
            (*self.render_resource_manager)
                .get_value_mut(self.id, "ran from drop")
                .as_mut()
                .unwrap()
                .ref_count -= 1;
        }
    }
}
impl<T> Hash for GenericStateRefMut<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state)
    }
}
