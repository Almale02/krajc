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
use tokio::sync::{RwLock, RwLockMappedWriteGuard, RwLockReadGuard, TryLockError};
use uuid::Uuid;

use crate::{engine_runtime::EngineRuntime, typed_addr::dupe, Lateinit};

use super::asset_loaders::file_resource_loader::SendEngineRuntime;

pub struct AssetManager {
    pub engine: Lateinit<&'static mut EngineRuntime>,
    pub engine_locked: Lateinit<SendEngineRuntime>,
    pub assets: HashMap<Uuid, Arc<RwLock<AssetEntrie>>>,
    pub main_tx: flume::Sender<(
        &'static Arc<RwLock<AssetEntrie>>,
        Box<dyn AssetLoader<Output = Box<dyn Any + Send>>>,
    )>,
    pub thread_rx: flume::Receiver<(
        &'static Arc<RwLock<AssetEntrie>>,
        Box<dyn AssetLoader<Output = Box<dyn Any + Send>>>,
    )>,
    //pub main_rx: flume::Receiver<(Uuid, Box<dyn Any + Send>)>,
    //pub thread_tx: flume::Sender<(Uuid, Box<dyn Any + Send>)>,
}
pub struct AssetEntrie {
    pub loaded: bool,
    pub asset: Option<Box<dyn Any + Send>>,
}
unsafe impl Send for AssetEntrie {}
unsafe impl Sync for AssetEntrie {}

impl AssetEntrie {
    pub fn new() -> Self {
        Self {
            loaded: false,
            asset: None,
        }
    }
}
impl std::fmt::Debug for AssetEntrie {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.loaded.fmt(f)
    }
}
pub struct AssetEntrieTyped<T> {
    pub loaded: bool,
    pub asset: Option<T>,
}

impl<T> AssetEntrieTyped<T> {
    pub fn new() -> Self {
        Self {
            loaded: false,
            asset: None,
        }
    }
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
            engine_locked: Default::default(), //main_rx,
                                               //thread_tx,
        }
    }

    pub fn load_resource<T: AssetLoader<Output = Box<dyn Any + Send>> + 'static>(
        &mut self,
        mut loader: T,
    ) -> AssetHandle<T> {
        let uuid = Uuid::new_v4();

        loader.set_engine((*self.engine_locked).clone());

        self.assets
            .insert(uuid, Arc::new(RwLock::new(AssetEntrie::new())));

        self.main_tx
            .send((dupe(self).assets.get(&uuid).unwrap(), Box::new(loader)))
            .unwrap();
        AssetHandle::new(uuid, dupe(self))
    }
    pub fn load_resource_bulk<T: AssetLoader<Output = Box<dyn Any + Send>> + 'static>(
        &mut self,
        loaders: Vec<T>,
    ) -> Vec<AssetHandle<T>> {
        loaders
            .into_iter()
            .map(|mut loader| {
                let uuid = Uuid::new_v4();
                loader.set_engine((*self.engine_locked).clone());

                self.assets
                    .insert(uuid, Arc::new(RwLock::new(AssetEntrie::new())));

                self.main_tx
                    .send((dupe(self).assets.get(&uuid).unwrap(), Box::new(loader)))
                    .unwrap();
                AssetHandle::new(uuid, dupe(self))
            })
            .collect()
    }
    pub fn load_resource_bulk_untype(
        &mut self,
        loaders: Vec<impl AssetLoader<Output = Box<dyn Any + Send>> + 'static>,
    ) -> Vec<AssetHandleUntype> {
        loaders
            .into_iter()
            .map(|mut loader| {
                let uuid = Uuid::new_v4();
                loader.set_engine((*self.engine_locked).clone());

                self.assets
                    .insert(uuid, Arc::new(RwLock::new(AssetEntrie::new())));

                self.main_tx
                    .send((dupe(self).assets.get(&uuid).unwrap(), Box::new(loader)))
                    .unwrap();
                AssetHandleUntype::new(uuid, dupe(self))
            })
            .collect()
    }
}

impl Default for AssetManager {
    fn default() -> Self {
        Self::new()
    }
}
pub struct AssetHandle<T> {
    pub uuid: Uuid,
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

impl<T: 'static> AssetHandle<T> {
    pub fn new(uuid: Uuid, manager: &'static mut AssetManager) -> Self {
        Self {
            uuid,
            manager,
            _p: PhantomData,
        }
    }
    pub async fn is_loaded(&self) -> bool {
        self.manager
            .assets
            .get(&self.uuid)
            .unwrap()
            .read()
            .await
            .loaded
    }
    pub async fn get(&self) -> RwLockReadGuard<T> {
        let a: tokio::sync::RwLockReadGuard<AssetEntrie> =
            self.manager.assets.get(&self.uuid).unwrap().read().await;
        tokio::sync::RwLockReadGuard::<'_, AssetEntrie>::map(a, |a| {
            a.asset.as_ref().unwrap().downcast_ref::<T>().unwrap()
        })
    }
    pub async fn get_checked(&self) -> Option<RwLockReadGuard<T>> {
        match self.is_loaded().await {
            true => Some(self.get().await),
            false => None,
        }
    }

    pub async fn get_mut(&mut self) -> RwLockMappedWriteGuard<T> {
        let a = self
            .manager
            .assets
            .get_mut(&self.uuid)
            .unwrap()
            .write()
            .await;
        let a = tokio::sync::RwLockWriteGuard::<'_, AssetEntrie>::map(a, |a| {
            a.asset.as_mut().unwrap().downcast_mut::<T>().unwrap()
        });
        a
    }
    pub async fn get_mut_checked(&mut self) -> Option<RwLockMappedWriteGuard<T>> {
        match self.is_loaded().await {
            true => Some(self.get_mut().await),
            false => None,
        }
    }
    pub fn is_loaded_blocking(&self) -> bool {
        self.manager
            .assets
            .get(&self.uuid)
            .unwrap()
            .blocking_read()
            .loaded
    }
    pub fn get_blocking(&self) -> RwLockReadGuard<T> {
        let a: tokio::sync::RwLockReadGuard<AssetEntrie> =
            self.manager.assets.get(&self.uuid).unwrap().blocking_read();
        tokio::sync::RwLockReadGuard::<'_, AssetEntrie>::map(a, |a| {
            a.asset.as_ref().unwrap().downcast_ref::<T>().unwrap()
        })
    }
    pub fn get_checked_blocking(&self) -> Option<RwLockReadGuard<T>> {
        match self.is_loaded_blocking() {
            true => Some(self.get_blocking()),
            false => None,
        }
    }

    pub fn get_mut_blocking(&mut self) -> RwLockMappedWriteGuard<T> {
        let a = self
            .manager
            .assets
            .get_mut(&self.uuid)
            .unwrap()
            .blocking_write();
        tokio::sync::RwLockWriteGuard::<'_, AssetEntrie>::map(a, |a| {
            a.asset.as_mut().unwrap().downcast_mut::<T>().unwrap()
        })
    }
    pub fn get_mut_checked_blocking(&mut self) -> Option<RwLockMappedWriteGuard<T>> {
        match self.is_loaded_blocking() {
            true => Some(self.get_mut_blocking()),
            false => None,
        }
    }
    pub fn try_is_loaded(&self) -> Result<bool, TryLockError> {
        let a = self.manager.assets.get(&self.uuid).unwrap().try_read()?;
        Ok(a.loaded)
    }
    pub fn try_get(&self) -> Result<RwLockReadGuard<T>, TryLockError> {
        let a: tokio::sync::RwLockReadGuard<AssetEntrie> =
            self.manager.assets.get(&self.uuid).unwrap().try_read()?;
        Ok(tokio::sync::RwLockReadGuard::<'_, AssetEntrie>::map(
            a,
            |a| a.asset.as_ref().unwrap().downcast_ref::<T>().unwrap(),
        ))
    }
    pub fn try_get_checked(&self) -> Result<Option<RwLockReadGuard<T>>, TryLockError> {
        match self.is_loaded_blocking() {
            true => Ok(Some(self.try_get()?)),
            false => Ok(None),
        }
    }

    pub fn try_get_mut(&mut self) -> Result<RwLockMappedWriteGuard<T>, TryLockError> {
        let a = self
            .manager
            .assets
            .get_mut(&self.uuid)
            .unwrap()
            .try_write()?;
        let res = tokio::sync::RwLockWriteGuard::<'_, AssetEntrie>::map(a, |a| {
            a.asset.as_mut().unwrap().downcast_mut::<T>().unwrap()
        });
        Ok(res)
    }
    pub fn try_get_mut_checked(&mut self) -> Option<RwLockMappedWriteGuard<T>> {
        match self.is_loaded_blocking() {
            true => Some(self.get_mut_blocking()),
            false => None,
        }
    }
}

impl<T: 'static> Future for AssetHandle<T> {
    type Output = Self;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.try_is_loaded() {
            Ok(x) => match x {
                true => Poll::Ready(self.clone()),
                false => {
                    cx.waker().wake_by_ref();
                    Poll::Pending
                }
            },
            Err(_) => {
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
    pub fn try_is_loaded(&self) -> Result<bool, TryLockError> {
        let a = self.manager.assets.get(&self.uuid).unwrap().try_read()?;
        Ok(a.loaded)
    }
}

impl Future for AssetHandleUntype {
    type Output = Self;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.try_is_loaded() {
            Ok(x) => match x {
                true => Poll::Ready(self.clone()),
                false => {
                    cx.waker().wake_by_ref();
                    Poll::Pending
                }
            },
            Err(_) => {
                cx.waker().wake_by_ref();
                Poll::Pending
            }
        }
    }
}

pub trait AssetLoader: Unpin + Send + Future<Output = Box<dyn Any + Send>> {
    fn set_engine(&mut self, engine: SendEngineRuntime);
}
pub struct SendWrapper<T: 'static> {
    pub value: &'static mut T,
}

impl<T: 'static> SendWrapper<T> {
    pub fn new(value: &'static mut T) -> Self {
        Self { value }
    }
}

impl<T> std::ops::Deref for SendWrapper<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &*self.value
    }
}
impl<T> std::ops::DerefMut for SendWrapper<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.value
    }
}
unsafe impl<T> Send for SendWrapper<T> {}
unsafe impl<T> Sync for SendWrapper<T> {}
