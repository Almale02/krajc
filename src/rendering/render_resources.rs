use std::{
    any::Any,
    collections::{HashMap, HashSet},
    marker::PhantomData,
    path::Path,
    pin::Pin,
    task::{Context, Poll},
};

use async_trait::async_trait;
use futures::{future::BoxFuture, stream::FuturesUnordered, Future, FutureExt};
use tokio::fs;
use uuid::Uuid;

use crate::typed_addr::dupe;

pub struct RenderResourceManager {
    pub resources: HashMap<Uuid, Box<dyn Any + Send>>,
    pub main_tx: flume::Sender<(Uuid, Box<dyn ResourceLoader>)>,
    pub thread_rx: flume::Receiver<(Uuid, Box<dyn ResourceLoader>)>,
    pub main_rx: flume::Receiver<(Uuid, Box<dyn Any + Send>)>,
    pub thread_tx: flume::Sender<(Uuid, Box<dyn Any + Send>)>,
}

pub struct ResourceHandle<T> {
    uuid: Uuid,
    manager: &'static mut RenderResourceManager,
    _p: PhantomData<T>,
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

impl RenderResourceManager {
    pub fn new() -> Self {
        let (main_tx, thread_rx) = flume::unbounded();
        let (thread_tx, main_rx) = flume::unbounded();
        Self {
            resources: HashMap::default(),
            main_tx,
            thread_rx,
            main_rx,
            thread_tx,
        }
    }

    pub fn load_resource<T>(&mut self, loader: Box<dyn ResourceLoader>) -> ResourceHandle<T> {
        let uuid = Uuid::new_v4();

        self.main_tx.send((uuid, loader)).unwrap();

        ResourceHandle::new(uuid, dupe(self))
    }
}

impl Default for RenderResourceManager {
    fn default() -> Self {
        Self::new()
    }
}

pub trait ResourceLoader: Send {
    fn start_loading(&mut self) -> BoxFuture<'static, Box<dyn Any + Send>>;
    fn start_loading_blocking(&mut self) -> Box<dyn Any + Send>;
}

pub struct FileResourceLoader<T: FileLoadable + 'static> {
    pub path: String,
    _p: PhantomData<T>,
}

impl<T: FileLoadable + Send + 'static> ResourceLoader for FileResourceLoader<T> {
    fn start_loading(&mut self) -> BoxFuture<'static, Box<dyn Any + Send>> {
        let path = self.path.clone();

        async move {
            let bytes = fs::read(path).await;
            let mut load_provider = T::default();

            Box::new(load_provider.process_bytes(bytes)) as Box<dyn Any + Send>
        }
        .boxed()
    }

    fn start_loading_blocking(&mut self) -> Box<dyn Any + Send> {
        let path = self.path.clone();

        let bytes = std::fs::read(path);
        let mut load_provider = T::default();

        Box::new(load_provider.process_bytes(bytes)) as Box<dyn Any + Send>
    }
}

pub trait FileLoadable: Default {
    type FinalResource: Send;
    fn process_bytes(&mut self, file: std::io::Result<Vec<u8>>) -> Self::FinalResource;
}

impl<T> Unpin for FileResourceLoader<T> where T: FileLoadable + Send + 'static {}
