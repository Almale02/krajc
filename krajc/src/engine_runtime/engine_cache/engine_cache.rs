use std::{any::Any, collections::HashMap};

pub struct EngineCache {
    // zero is an invalid id
    pub data: HashMap<String, Box<dyn Any>>,
}

impl EngineCache {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }
    pub fn cache<'cache, T: 'static>(
        &'cache mut self,
        new_id: &'static str,
        data: impl FnOnce() -> T,
    ) -> &'cache mut T {
        match self.data.contains_key(new_id) {
            true => self.data.get_mut(new_id).unwrap().downcast_mut().unwrap(),
            false => {
                self.data.insert(new_id.to_owned(), Box::new(data()));

                self.data
                    .get_mut(&new_id.to_owned())
                    .unwrap()
                    .downcast_mut()
                    .unwrap()
            }
        }
    }
}

impl Default for EngineCache {
    fn default() -> Self {
        Self::new()
    }
}
