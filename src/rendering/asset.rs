use std::{
    any::Any,
    boxed::Box,
    cell::UnsafeCell,
    collections::HashMap,
    hash::Hash,
    marker::{PhantomData, Send},
    pin::Pin,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    task::{Context, Poll},
};

use bevy_ecs::{component::Component, ptr::UnsafeCellDeref};
use futures::Future;
use uuid::Uuid;

use crate::{
    engine_runtime::{
        schedule_manager::system_params::system_resource::EngineResource, EngineRuntime,
    },
    typed_addr::dupe,
    FromEngine, Lateinit,
};

use super::asset_loaders::file_resource_loader::SendEngineRuntime;

pub struct AssetManager {
    pub engine: Lateinit<&'static mut EngineRuntime>,
    pub engine_locked: Lateinit<SendEngineRuntime>,
    pub assets: HashMap<Uuid, Arc<AssetEntrie>>,
    pub main_tx: flume::Sender<(
        Arc<AssetEntrie>,
        Box<dyn AssetLoader<Output = Box<dyn Any + Send>>>,
    )>,
    pub thread_rx: flume::Receiver<(
        Arc<AssetEntrie>,
        Box<dyn AssetLoader<Output = Box<dyn Any + Send>>>,
    )>,
    pub thread_main_exec_tx: flume::Sender<Box<dyn Fn()>>, //pub main_rx: flume::Receiver<(Uuid, Box<dyn Any + Send>)>,
    pub main_exec_rx: flume::Receiver<Box<dyn Fn()>>, //pub thread_tx: flume::Sender<(Uuid, Box<dyn Any + Send>)>,
    pub loaded_callback_tx: flume::Sender<(
        AssetHandleUntype,
        fn(),
        fn(AssetHandleUntype, &'static mut EngineRuntime),
    )>,
    pub loaded_callback_rx: flume::Receiver<(
        AssetHandleUntype,
        fn(),
        fn(AssetHandleUntype, &'static mut EngineRuntime),
    )>,
}
impl EngineResource for AssetManager {
    fn get(engine: &'static mut EngineRuntime) -> &'static Self {
        &engine.asset_manager
    }
    fn get_mut(engine: &'static mut EngineRuntime) -> &'static mut Self {
        &mut engine.asset_manager
    }
    fn get_no_init(engine: &'static EngineRuntime) -> &'static Self {
        &engine.asset_manager
    }
}
unsafe impl Send for AssetManager {}
pub struct AssetEntrie {
    pub loaded: AtomicBool,
    /// an asset could only be modifed if it is marked as *unloaded*, if it is unloaded then it *cannot* be read,
    /// the library which allow the users to use AssetHandles need to ensure that if the asset is marked as unloaded, then it wont run their code that could use those assets.
    /// and the users of those libraries needs to register their [`AssetHandle`s] to the library as an [AssetHandleUntype]
    /// take a look at [super::draw_pass::DrawPass] for example of this
    pub asset: UnsafeCell<Box<dyn Any + Send>>,
    pub callbacks: Vec<fn(AssetHandleUntype, &'static mut EngineRuntime)>,
    pub uuid: Uuid,
}
unsafe impl Send for AssetEntrie {}
unsafe impl Sync for AssetEntrie {}

impl AssetEntrie {
    pub fn new(
        uuid: Uuid,
        callbacks: Vec<fn(AssetHandleUntype, &'static mut EngineRuntime)>,
    ) -> Self {
        Self {
            loaded: false.into(),
            asset: UnsafeCell::new(Box::new(0_u8)),
            callbacks,
            uuid,
        }
    }
    /// asset could only be modified if it is marked as unloaded, this is to prevent reading the value while it is being modified.
    pub fn get_mut(&self) -> Option<&'static mut Box<dyn Any + Send>> {
        if self.loaded.load(Ordering::SeqCst) {
            println!("asset is not marked as being unloaded!");
            return None;
        }
        Some(unsafe { dupe(self).asset.deref_mut() })
    }
}

impl Default for AssetEntrie {
    fn default() -> Self {
        Self::new(Uuid::default(), Vec::default())
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

impl<T> Default for AssetEntrieTyped<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl AssetManager {
    pub fn new() -> Self {
        let (main_tx, thread_rx) = flume::unbounded();
        //let (thread_tx, main_rx) = flume::unbounded();

        let (thread_main_exec_tx, main_exec_rx) = flume::unbounded();
        let (loaded_callback_tx, loaded_callback_rx) = flume::unbounded();
        Self {
            engine: Lateinit::default(),
            assets: HashMap::default(),
            main_tx,
            thread_rx,
            engine_locked: Default::default(),
            thread_main_exec_tx,
            main_exec_rx,
            loaded_callback_tx,
            loaded_callback_rx,
        }
    }

    pub fn load_resource<T: AssetLoader<Output = Box<dyn Any + Send>> + FinalAsset + 'static>(
        &mut self,
        mut loader: T,
        callbacks: Vec<fn(AssetHandleUntype, &'static mut EngineRuntime)>,
    ) -> AssetHandle<<T as FinalAsset>::FinalAsset> {
        let uuid = Uuid::new_v4();

        loader.set_engine((*self.engine_locked).clone());
        loader.set_thread_main_exec(self.thread_main_exec_tx.clone());

        self.assets
            .insert(uuid, Arc::new(AssetEntrie::new(uuid, callbacks)));

        self.main_tx
            .send((
                dupe(self).assets.get(&uuid).unwrap().clone(),
                Box::new(loader),
            ))
            .unwrap();
        AssetHandle::new(uuid, dupe(self))
    }
    /*pub fn load_resource_bulk<T: AssetLoader<Output = Box<dyn Any + Send>> + 'static>(
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
    }*/
}

impl Default for AssetManager {
    fn default() -> Self {
        Self::new()
    }
}
#[derive(Component)]
pub struct AssetHandle<T> {
    pub uuid: Uuid,
    manager: SendWrapper<&'static mut AssetManager>,
    _p: PhantomData<T>,
}
impl<T> Hash for AssetHandle<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.uuid.hash(state);
    }
}
impl<T> PartialEq for AssetHandle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.uuid.eq(&other.uuid)
    }
}
impl<T> Eq for AssetHandle<T> {}
impl<T> FromEngine for AssetHandle<T> {
    fn from_engine(engine: &'static mut EngineRuntime) -> Self {
        Self {
            uuid: Uuid::default(),
            manager: SendWrapper::new(&mut engine.asset_manager),
            _p: PhantomData,
        }
    }
}

impl<T> Clone for AssetHandle<T> {
    fn clone(&self) -> Self {
        Self {
            uuid: self.uuid,
            manager: SendWrapper::new(dupe(dupe(self.manager.value))),
            _p: PhantomData,
        }
    }
}

impl<T: 'static> AssetHandle<T> {
    pub fn as_untype(&self) -> AssetHandleUntype {
        AssetHandleUntype::new(self.uuid, dupe(self.manager.value))
    }
    pub fn new(uuid: Uuid, manager: &'static mut AssetManager) -> Self {
        Self {
            uuid,
            manager: SendWrapper { value: manager },
            _p: PhantomData,
        }
    }
    pub fn is_loaded(&self) -> Option<bool> {
        let a = self.manager.assets.get(&self.uuid);
        match a {
            Some(x) => Some(x.loaded.load(Ordering::SeqCst)),
            None => None,
        }

        /*.expect(format!("didnt find resource with {}", self.uuid).as_str())
        .loaded
        .load(Ordering::SeqCst)*/
    }

    /// this ___MUST___ be only used in __callbacks__
    pub unsafe fn get_unchecked(&self) -> Option<&'static T> {
        unsafe {
            dupe(self)
                .manager
                .assets
                .get(&self.uuid)
                .unwrap()
                .asset
                .deref()
                .downcast_ref()
        }
    }

    /// asset could only be modified if it is marked as unloaded, this is to prevent reading the value while it is being modified.
    pub fn get_mut(&mut self) -> Option<&'static mut T> {
        if !self.is_loaded().unwrap() {
            return None;
        }
        Some(unsafe {
            dupe(self)
                .manager
                .assets
                .get(&self.uuid)
                .unwrap()
                .asset
                .deref_mut()
                .downcast_mut()
                .unwrap()
        })
    }

    /// the library which allow the users to use AssetHandles need to ensure that if the asset is marked as unloaded,
    /// then it wont run their code that could use those assets.
    /// so it should be safae to unwrap this
    pub fn get(&self) -> Option<&'static T> {
        unsafe {
            match self.is_loaded() {
                Some(x) => match x {
                    true => match self.get_unchecked() {
                        Some(x) => Some(x),
                        None => None,
                    },
                    false => None,
                },
                None => None,
            }
        }
    }
}

impl<T: 'static> Future for AssetHandle<T> {
    type Output = Self;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.is_loaded().unwrap() {
            true => Poll::Ready(self.clone()),
            false => {
                cx.waker().wake_by_ref();
                Poll::Pending
            }
        }
    }
}

pub struct AssetHandleUntype {
    pub uuid: Uuid,
    manager: &'static mut AssetManager,
}

unsafe impl Send for AssetHandleUntype {}

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
        self.manager
            .assets
            .get(&self.uuid)
            .unwrap()
            .loaded
            .load(Ordering::SeqCst)
    }
    pub fn get_typed<T: 'static>(&self) -> AssetHandle<T> {
        AssetHandle::new(self.uuid, dupe(self).manager)
    }
}

impl Future for AssetHandleUntype {
    type Output = Self;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.is_loaded() {
            true => Poll::Ready(self.clone()),
            false => {
                cx.waker().wake_by_ref();
                Poll::Pending
            }
        }
    }
}
impl FromEngine for AssetHandleUntype {
    fn from_engine(engine: &'static mut EngineRuntime) -> Self {
        Self {
            uuid: Uuid::default(),
            manager: &mut engine.asset_manager,
        }
    }
}

pub trait AssetLoader: Unpin + Send + Future<Output = Box<dyn Any + Send>> {
    fn set_engine(&mut self, engine: SendEngineRuntime);
    fn set_thread_main_exec(&mut self, tx: flume::Sender<Box<dyn Fn()>>);
}
pub trait FinalAsset {
    type FinalAsset;
}

pub struct SendWrapper<T: 'static> {
    pub value: T,
}
impl<T: Default> Default for SendWrapper<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T: 'static> SendWrapper<T> {
    pub fn new(value: T) -> Self {
        Self { value }
    }
}

impl<T> std::ops::Deref for SendWrapper<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
impl<T> std::ops::DerefMut for SendWrapper<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}
unsafe impl<T> Send for SendWrapper<T> {}
unsafe impl<T> Sync for SendWrapper<T> {}
