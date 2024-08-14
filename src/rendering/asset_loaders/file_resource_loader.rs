use crate::engine_runtime::EngineRuntime;
use crate::rendering::asset::FinalAsset;
use crate::rendering::asset::SendWrapper;
use crate::rendering::managers::RenderManagerResource;
use crate::rendering::texture::texture::Texture;
use crate::typed_addr::dupe;
use crate::AssetLoader;
use crate::Lateinit;
use crate::Takeable;
use futures::future::BoxFuture;
use futures::task::Context;
use futures::task::Poll;
use futures::FutureExt;
use std::any::Any;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::fs;
use tokio::sync::RwLock;
use wgpu::ShaderModule;
use wgpu::ShaderModuleDescriptor;

enum FileResourceLoaderState {
    LoadingFile,
    RunningProcessor,
}

pub type SendEngineRuntime = Arc<RwLock<SendWrapper<&'static mut EngineRuntime>>>;
pub struct FileResourceLoader<
    T: FileLoadable + Future<Output = Box<dyn Any + Send>> + FinalAsset + Send + 'static,
> {
    pub path: &'static str,
    file_loader: BoxFuture<'static, tokio::io::Result<Vec<u8>>>,
    state: FileResourceLoaderState,
    processor: T,
    callbacks: SendWrapper<
        Vec<Takeable<Box<dyn Fn(&'static Box<dyn Any + Send>, &'static mut EngineRuntime)>>>,
    >,

    engine: Lateinit<SendEngineRuntime>,
    thread_main_exec_tx: Lateinit<flume::Sender<Box<dyn Fn()>>>,
}
impl<
        T: FileLoadable
            + 'static
            + Future<Output = Box<dyn Any + Send>>
            + FinalAsset
            + Send
            + Unpin
            + Default,
    > FinalAsset for FileResourceLoader<T>
{
    type FinalAsset = <T as FinalAsset>::FinalAsset;
}

impl<
        T: FileLoadable + 'static + Future<Output = Box<dyn Any + Send>> + FinalAsset + Send + Unpin,
    > FileResourceLoader<T>
{
    pub fn new(path: &'static str, file_loader: T) -> Self {
        Self {
            path,
            file_loader: fs::read(path).boxed(),
            state: FileResourceLoaderState::LoadingFile,
            processor: file_loader,
            engine: Default::default(),
            thread_main_exec_tx: Default::default(),
            callbacks: SendWrapper::default(),
        }
    }
}

impl<
        T: FileLoadable + Future<Output = Box<dyn Any + Send>> + Send + FinalAsset + Unpin + 'static,
    > AssetLoader for FileResourceLoader<T>
{
    fn set_engine(&mut self, engine: SendEngineRuntime) {
        self.engine.set(engine);
    }
    fn set_thread_main_exec(&mut self, tx: flume::Sender<Box<dyn Fn()>>) {
        self.thread_main_exec_tx.set(tx);
    }
}

impl<
        T: FileLoadable + Future<Output = Box<dyn Any + Send>> + 'static + FinalAsset + Send + Unpin,
    > Future for FileResourceLoader<T>
{
    type Output = T::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let engine = self.engine.get().clone();
        let this = unsafe { self.get_unchecked_mut() };
        this.processor.set_engine(engine);
        this.processor
            .set_thread_main_exec(this.thread_main_exec_tx.get().clone());

        match this.state {
            FileResourceLoaderState::LoadingFile => match this.file_loader.poll_unpin(cx) {
                Poll::Ready(x) => {
                    cx.waker().wake_by_ref();
                    this.processor.set_bytes(x);
                    this.state = FileResourceLoaderState::RunningProcessor;
                    Poll::Pending
                }
                Poll::Pending => {
                    cx.waker().wake_by_ref();
                    Poll::Pending
                }
            },
            FileResourceLoaderState::RunningProcessor => match this.processor.poll_unpin(cx) {
                Poll::Ready(x) => {
                    cx.waker().wake_by_ref();

                    Poll::Ready(x)
                }
                Poll::Pending => {
                    cx.waker().wake_by_ref();
                    Poll::Pending
                }
            },
        }
    }
}

pub trait FileLoadable {
    fn set_bytes(&mut self, file: std::io::Result<Vec<u8>>);
    fn set_engine(&mut self, engine: SendEngineRuntime);

    fn set_thread_main_exec(&mut self, tx: flume::Sender<Box<dyn Fn()>>);
}

/// could be use with FileResourceLoader
/// returns Vec<u8> of the file
#[derive(Default)]
pub struct RawFileLoader {
    pub bytes: Vec<u8>,
    engine: Lateinit<SendEngineRuntime>,
    _thread_main_exec_tx:
        Lateinit<flume::Sender<(Box<dyn Any + Send>, flume::Sender<Box<dyn Any + Send>>)>>,
}
impl RawFileLoader {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self {
            bytes,
            engine: Lateinit::default(),
            _thread_main_exec_tx: Lateinit::default(),
        }
    }
}

impl Future for RawFileLoader {
    type Output = Box<dyn Any + Send>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        cx.waker().wake_by_ref();
        Poll::Ready(Box::new(self.bytes.clone()))
    }
}

impl FileLoadable for RawFileLoader {
    fn set_bytes(&mut self, file: std::io::Result<Vec<u8>>) {
        self.bytes = file.unwrap();
    }

    fn set_engine(&mut self, engine: SendEngineRuntime) {
        self.engine.set(engine);
    }
    fn set_thread_main_exec(&mut self, _tx: flume::Sender<Box<dyn Fn()>>) {}
}

impl<T: FinalAsset> Unpin for FileResourceLoader<T> where
    T: FileLoadable + Send + Future<Output = Box<(dyn Any + Send + 'static)>> + 'static
{
}

/// could be use with FileResourceLoader
/// returns the ShaderModule
pub struct ShaderLoader {
    pub bytes: Vec<u8>,
    engine: Lateinit<SendEngineRuntime>,
    thread_main_exec_tx: Lateinit<flume::Sender<Box<dyn Fn()>>>,
    main_exec_callback_tx: flume::Sender<Box<dyn Any + Send>>,
    main_exec_callback_rx: flume::Receiver<Box<dyn Any + Send>>,
    state: ShaderLoaderState,
}

impl FinalAsset for ShaderLoader {
    type FinalAsset = ShaderModule;
}

impl Default for ShaderLoader {
    fn default() -> Self {
        ShaderLoader::new(Vec::default())
    }
}
impl ShaderLoader {
    pub fn new(bytes: Vec<u8>) -> Self {
        let (main_exec_callback_tx, main_exec_callback_rx) = flume::unbounded();
        Self {
            bytes,
            engine: Lateinit::default(),
            thread_main_exec_tx: Lateinit::default(),
            main_exec_callback_tx,
            main_exec_callback_rx,
            state: ShaderLoaderState::Start,
        }
    }
}

enum ShaderLoaderState {
    Start,
    WaitingForMain,
}
impl Future for ShaderLoader {
    type Output = Box<dyn Any + Send>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        cx.waker().wake_by_ref();
        let this = unsafe { self.get_unchecked_mut() };

        let engine = (*dupe(this).engine).try_write();

        if engine.is_err() {
            return Poll::Pending;
        }

        match &this.state {
            ShaderLoaderState::Start => {
                let engine = engine.unwrap();
                let bytes = this.bytes.clone();
                let tx = this.main_exec_callback_tx.clone();
                this.thread_main_exec_tx
                    .clone()
                    .send(Box::new(move || {
                        let render = engine.get_resource_no_init::<RenderManagerResource>();

                        let module = render.device.create_shader_module(ShaderModuleDescriptor {
                            label: Some("Shader from ShaderLoader"),
                            source: wgpu::ShaderSource::Wgsl(
                                String::from_utf8(bytes.clone()).unwrap().as_str().into(),
                            ),
                        });
                        tx.send(Box::new(module)).unwrap();
                    }))
                    .unwrap();
                this.state = ShaderLoaderState::WaitingForMain;
                cx.waker().wake_by_ref();
                Poll::Pending
            }
            ShaderLoaderState::WaitingForMain => match this.main_exec_callback_rx.try_recv() {
                Ok(module) => Poll::Ready(module),
                Err(_) => {
                    cx.waker().wake_by_ref();
                    Poll::Pending
                }
            },
        }
    }
}

impl FileLoadable for ShaderLoader {
    fn set_bytes(&mut self, file: std::io::Result<Vec<u8>>) {
        self.bytes = file.unwrap();
    }

    fn set_engine(&mut self, engine: SendEngineRuntime) {
        self.engine.set(engine);
    }
    fn set_thread_main_exec(&mut self, tx: flume::Sender<Box<dyn Fn()>>) {
        self.thread_main_exec_tx.set(tx);
    }
}
enum TextureLoaderState {
    Start,
    WaitingForMain,
}

pub struct TextureLoader {
    bytes: Vec<u8>,
    engine: Lateinit<SendEngineRuntime>,
    thread_main_exec_tx: Lateinit<flume::Sender<Box<dyn Fn()>>>,
    main_exec_callback_tx: flume::Sender<Box<dyn Any + Send>>,
    main_exec_callback_rx: flume::Receiver<Box<dyn Any + Send>>,
    state: TextureLoaderState,
}

impl TextureLoader {
    pub fn new(bytes: Vec<u8>) -> Self {
        let (main_exec_callback_tx, main_exec_callback_rx) = flume::unbounded();
        Self {
            bytes,
            engine: Lateinit::default(),
            thread_main_exec_tx: Lateinit::default(),
            main_exec_callback_tx,
            main_exec_callback_rx,
            state: TextureLoaderState::Start,
        }
    }
}
impl Default for TextureLoader {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}
impl FileLoadable for TextureLoader {
    fn set_bytes(&mut self, file: std::io::Result<Vec<u8>>) {
        self.bytes = file.unwrap();
    }

    fn set_engine(&mut self, engine: SendEngineRuntime) {
        self.engine.set(engine);
    }
    fn set_thread_main_exec(&mut self, tx: flume::Sender<Box<dyn Fn()>>) {
        self.thread_main_exec_tx.set(tx);
    }
}

impl FinalAsset for TextureLoader {
    type FinalAsset = Texture;
}

impl Future for TextureLoader {
    type Output = Box<dyn Any + Send>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        cx.waker().wake_by_ref();
        let this = unsafe { self.get_unchecked_mut() };

        let engine = (*dupe(this).engine).try_write();

        if engine.is_err() {
            return Poll::Pending;
        }

        match &this.state {
            TextureLoaderState::Start => {
                let engine = engine.unwrap();
                let bytes = this.bytes.clone();

                let tx = this.main_exec_callback_tx.clone();
                this.thread_main_exec_tx
                    .clone()
                    .send(Box::new(move || {
                        let render = engine.get_resource_no_init::<RenderManagerResource>();

                        let mut texture = Texture::from_bytes(
                            &render.device,
                            &render.queue,
                            &bytes,
                            "Texture created in AssetLoader",
                        )
                        .unwrap();
                        texture
                            .texture_bind_group
                            .set(texture.get_texture_bind_group(&render.device));
                        tx.send(Box::new(texture)).unwrap();
                    }))
                    .unwrap();
                this.state = TextureLoaderState::WaitingForMain;
                cx.waker().wake_by_ref();
                Poll::Pending
            }
            TextureLoaderState::WaitingForMain => match this.main_exec_callback_rx.try_recv() {
                Ok(module) => Poll::Ready(module),
                Err(_) => {
                    cx.waker().wake_by_ref();
                    Poll::Pending
                }
            },
        }
    }
}
pub struct MemoryAsset<T> {
    data: Takeable<T>,
}

impl<T> MemoryAsset<T> {
    pub fn new(data: T) -> Self {
        Self {
            data: Takeable::new(data),
        }
    }
}
impl<T> FinalAsset for MemoryAsset<T> {
    type FinalAsset = T;
}
impl<T: Send + Unpin + 'static> Future for MemoryAsset<T> {
    type Output = Box<dyn Any + Send>;

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        Poll::Ready(Box::new(self.get_mut().data.take().unwrap()))
    }
}

impl<T: Send + Unpin + 'static> AssetLoader for MemoryAsset<T> {
    fn set_engine(&mut self, _engine: SendEngineRuntime) {
        //
    }

    fn set_thread_main_exec(&mut self, _tx: flume::Sender<Box<dyn Fn()>>) {
        //
    }
}
