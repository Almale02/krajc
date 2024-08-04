use std::{
    any::Any,
    boxed::Box,
    collections::HashMap,
    marker::{PhantomData, Send},
    pin::Pin,
    task::{Context, Poll},
};

use futures::{future::BoxFuture, Future, FutureExt};
use tokio::fs;
use uuid::Uuid;

use crate::{engine_runtime::EngineRuntime, typed_addr::dupe, Lateinit};

pub struct RenderResourceManager {
    pub engine: Lateinit<&'static mut EngineRuntime>,
    pub resources: HashMap<Uuid, Box<dyn Any + Send>>,
    pub main_tx: flume::Sender<(Uuid, Box<dyn ResourceLoader<Output = Box<dyn Any + Send>>>)>,
    pub thread_rx: flume::Receiver<(Uuid, Box<dyn ResourceLoader<Output = Box<dyn Any + Send>>>)>,
    pub main_rx: flume::Receiver<(Uuid, Box<dyn Any + Send>)>,
    pub thread_tx: flume::Sender<(Uuid, Box<dyn Any + Send>)>,
}

impl RenderResourceManager {
    pub fn new() -> Self {
        let (main_tx, thread_rx) = flume::unbounded();
        let (thread_tx, main_rx) = flume::unbounded();
        Self {
            engine: Lateinit::default(),
            resources: HashMap::default(),
            main_tx,
            thread_rx,
            main_rx,
            thread_tx,
        }
    }

    pub fn load_resource<T: ResourceLoader<Output = Box<dyn Any + Send>> + 'static>(
        &mut self,
        mut loader: T,
    ) -> ResourceHandle<T> {
        let uuid = Uuid::new_v4();
        loader.set_engine(*dupe(self).engine);

        self.main_tx.send((uuid, Box::new(loader))).unwrap();

        ResourceHandle::new(uuid, dupe(self))
    }
    pub fn update(&mut self) {
        while let Ok((uuid, res)) = self.main_rx.try_recv() {
            self.resources.insert(uuid, res);
        }
    }
}

impl Default for RenderResourceManager {
    fn default() -> Self {
        Self::new()
    }
}
pub struct ResourceHandle<T> {
    uuid: Uuid,
    manager: &'static mut RenderResourceManager,
    _p: PhantomData<T>,
}

impl<T> Clone for ResourceHandle<T> {
    fn clone(&self) -> Self {
        Self {
            uuid: self.uuid,
            manager: dupe(self.manager),
            _p: PhantomData,
        }
    }
}

impl<T> ResourceHandle<T> {
    pub fn new(uuid: Uuid, manager: &'static mut RenderResourceManager) -> Self {
        Self {
            uuid,
            manager,
            _p: PhantomData,
        }
    }
    pub fn is_loaded(&self) -> bool {
        self.manager.resources.contains_key(&self.uuid)
    }
    pub fn get(&self) -> &'static T {
        dupe(self)
            .manager
            .resources
            .get(&self.uuid)
            .unwrap()
            .downcast_ref()
            .unwrap()
    }
    pub fn get_checked(&self) -> Option<&'static T> {
        match self.is_loaded() {
            true => Some(self.get()),
            false => None,
        }
    }

    pub fn get_mut(&self) -> &'static mut T {
        dupe(self)
            .manager
            .resources
            .get_mut(&self.uuid)
            .unwrap()
            .downcast_mut()
            .unwrap()
    }
    pub fn get_mut_checked(&self) -> Option<&'static mut T> {
        match self.is_loaded() {
            true => Some(self.get_mut()),
            false => None,
        }
    }
}

impl<T> Future for ResourceHandle<T> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.is_loaded() {
            true => Poll::Ready(()),
            false => {
                cx.waker().wake_by_ref();
                Poll::Pending
            }
        }
    }
}

pub struct ResourceHandleUntype {
    uuid: Uuid,
    manager: &'static mut RenderResourceManager,
}

impl Clone for ResourceHandleUntype {
    fn clone(&self) -> Self {
        Self {
            uuid: self.uuid,
            manager: dupe(self.manager),
        }
    }
}

impl ResourceHandleUntype {
    pub fn new(uuid: Uuid, manager: &'static mut RenderResourceManager) -> Self {
        Self { uuid, manager }
    }
    pub fn is_loaded(&self) -> bool {
        self.manager.resources.contains_key(&self.uuid)
    }
}

pub trait ResourceLoader: Unpin + Send + Future<Output = Box<dyn Any + Send>> {
    fn set_engine(&mut self, engine: &'static mut EngineRuntime);
}
