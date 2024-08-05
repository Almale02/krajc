use std::{
    any::Any,
    boxed::Box,
    collections::HashMap,
    marker::{PhantomData, Send},
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use futures::{future::BoxFuture, Future, FutureExt};
use tokio::fs;
use uuid::Uuid;

use crate::{engine_runtime::EngineRuntime, typed_addr::dupe, Lateinit};

pub struct AssetManager {
    pub engine: Lateinit<&'static mut EngineRuntime>,
    pub assets: HashMap<Uuid, Arc<Box<dyn Any + Send>>>,
    pub main_tx: flume::Sender<(
        Arc<Box<dyn Any + Send>>,
        Box<dyn AssetLoader<Output = Box<dyn Any + Send>>>,
    )>,
    pub thread_rx: flume::Receiver<(
        Arc<Box<dyn Any + Send>>,
        Box<dyn AssetLoader<Output = Box<dyn Any + Send>>>,
    )>,
    //pub main_rx: flume::Receiver<(Uuid, Box<dyn Any + Send>)>,
    //pub thread_tx: flume::Sender<(Uuid, Box<dyn Any + Send>)>,
}

impl AssetManager {
    pub fn new() -> Self {
        let (main_tx, thread_rx) = flume::unbounded();
        //let (thread_tx, main_rx) = flume::unbounded();
        Self {
            engine: Lateinit::default(),
            assets: HashMap::default(),
            main_tx,
            thread_rx,
            //main_rx,
            //thread_tx,
        }
    }

    pub fn load_resource<T: AssetLoader<Output = Box<dyn Any + Send>> + 'static>(
        &mut self,
        mut loader: T,
    ) -> AssetHandle<T> {
        let uuid = Uuid::new_v4();
        loader.set_engine(*dupe(self).engine);

        self.assets.insert(uuid, Arc::new(Box::new(0_u8)));
        let asset_ref = self.assets.get(&uuid).unwrap();

        self.main_tx
            .send((Arc::clone(asset_ref), Box::new(loader)))
            .unwrap();

        AssetHandle::new(uuid, dupe(self))
    }
}

impl Default for AssetManager {
    fn default() -> Self {
        Self::new()
    }
}
pub struct AssetHandle<T> {
    uuid: Uuid,
    manager: &'static mut AssetManager,
    _p: PhantomData<T>,
}

impl<T> Clone for AssetHandle<T> {
    fn clone(&self) -> Self {
        Self {
            uuid: self.uuid,
            manager: dupe(self.manager),
            _p: PhantomData,
        }
    }
}

impl<T> AssetHandle<T> {
    pub fn new(uuid: Uuid, manager: &'static mut AssetManager) -> Self {
        Self {
            uuid,
            manager,
            _p: PhantomData,
        }
    }
    pub fn is_loaded(&self) -> bool {
        self.manager.assets.contains_key(&self.uuid)
    }
    pub fn get(&self) -> &'static T {
        dupe(self)
            .manager
            .assets
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
            .assets
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

impl<T> Future for AssetHandle<T> {
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

pub struct AssetHandleUntype {
    uuid: Uuid,
    manager: &'static mut AssetManager,
}

impl Clone for AssetHandleUntype {
    fn clone(&self) -> Self {
        Self {
            uuid: self.uuid,
            manager: dupe(self.manager),
        }
    }
}

impl AssetHandleUntype {
    pub fn new(uuid: Uuid, manager: &'static mut AssetManager) -> Self {
        Self { uuid, manager }
    }
    pub fn is_loaded(&self) -> bool {
        self.manager.assets.contains_key(&self.uuid)
    }
}

pub trait AssetLoader: Unpin + Send + Future<Output = Box<dyn Any + Send>> {
    fn set_engine(&mut self, engine: &'static mut EngineRuntime);
}
