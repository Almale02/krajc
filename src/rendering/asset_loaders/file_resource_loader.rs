use crate::engine_runtime::EngineRuntime;
use crate::rendering::asset::SendWrapper;
use crate::rendering::managers::RenderManagerResource;
use crate::typed_addr::dupe;
use crate::typed_addr::TypedAddr;
use crate::AssetLoader;
use crate::Lateinit;
use futures::future::BoxFuture;
use futures::task::Context;
use futures::task::Poll;
use futures::task::UnsafeFutureObj;
use futures::FutureExt;
use rapier3d::crossbeam::atomic::AtomicCell;
use std::any::Any;
use std::cell::Cell;
use std::future::Future;
use std::ops::Deref;
use std::pin::Pin;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use tokio::fs;
use tokio::sync::RwLock;
use wgpu::ShaderModuleDescriptor;

pub type SendEngineRuntime = Arc<RwLock<SendWrapper<EngineRuntime>>>;
pub struct FileResourceLoader<T: FileLoadable + Future<Output = Box<dyn Any + Send>> + 'static> {
    pub path: &'static str,
    file_loader: BoxFuture<'static, tokio::io::Result<Vec<u8>>>,
    loaded_file: bool,
    processor: T,
    engine: Lateinit<SendEngineRuntime>,
    thread_main_exec_tx:
        Lateinit<flume::Sender<(Box<dyn Any + Send>, Box<dyn Fn(Box<dyn Any + Send>)>)>>,
}

impl<T: FileLoadable + 'static + Future<Output = Box<dyn Any + Send>> + Send + Unpin + Default>
    FileResourceLoader<T>
{
    pub fn new(path: &'static str) -> Self {
        //AtomicU64
        Self {
            path,
            file_loader: fs::read(path).boxed(),
            loaded_file: false,
            processor: T::default(),
            engine: Default::default(),
            thread_main_exec_tx: Default::default(),
        }
    }
}

impl<T: FileLoadable + Future<Output = Box<dyn Any + Send>> + Send + Unpin + 'static> AssetLoader
    for FileResourceLoader<T>
{
    fn set_engine(&mut self, engine: SendEngineRuntime) {
        self.engine.set(engine);
    }
    fn set_thread_main_exec(
        &mut self,
        tx: flume::Sender<(Box<dyn Any + Send>, Box<dyn Fn(Box<dyn Any + Send>)>)>,
    ) {
        self.thread_main_exec_tx.set(tx);
    }
}

impl<T: FileLoadable + Future<Output = Box<dyn Any + Send>> + 'static + Send + Unpin> Future
    for FileResourceLoader<T>
{
    type Output = T::Output;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let engine = self.engine.get().clone();
        self.processor.set_engine(engine);

        match self.loaded_file {
            true => {
                match self.processor.poll_unpin(cx) {
                    Poll::Ready(x) => Poll::Ready(x),
                    Poll::Pending => {
                        cx.waker().wake_by_ref();
                        Poll::Pending
                    }
                }
                //
            }
            false => match self.file_loader.poll_unpin(cx) {
                Poll::Ready(x) => {
                    cx.waker().wake_by_ref();
                    self.processor.set_bytes(x);
                    self.loaded_file = true;

                    Poll::Pending
                }
                Poll::Pending => {
                    cx.waker().wake_by_ref();
                    Poll::Pending
                }
            },
        }
    }
}

pub trait FileLoadable: Default {
    fn set_bytes(&mut self, file: std::io::Result<Vec<u8>>);
    fn set_engine(&mut self, engine: SendEngineRuntime);

    fn set_thread_main_exec(
        &mut self,
        tx: flume::Sender<(Box<dyn Any + Send>, Box<dyn Fn(Box<dyn Any + Send>)>)>,
    );
}

/// could be use with FileResourceLoader
/// returns Vec<u8> of the file
#[derive(Default)]
pub struct RawFileLoader {
    pub bytes: Vec<u8>,
    engine: Lateinit<SendEngineRuntime>,
    thread_main_exec_tx:
        Lateinit<flume::Sender<(Box<dyn Any + Send>, flume::Sender<Box<dyn Any + Send>>)>>,
}
impl RawFileLoader {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self {
            bytes,
            engine: Lateinit::default(),
            thread_main_exec_tx: Lateinit::default(),
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

    fn set_thread_main_exec(
        &mut self,
        tx: flume::Sender<(Box<dyn Any + Send>, Box<dyn Fn(Box<dyn Any + Send>)>)>,
    ) {
        //self.thread_main_exec_tx.set(tx);
    }
}

impl<T> Unpin for FileResourceLoader<T> where
    T: FileLoadable + Send + Future<Output = Box<(dyn Any + Send + 'static)>> + 'static
{
}

/// could be use with FileResourceLoader
/// returns the ShaderModule
pub struct ShaderLoader {
    pub bytes: Vec<u8>,
    engine: Lateinit<SendEngineRuntime>,
    thread_main_exec_tx:
        Lateinit<flume::Sender<(Box<dyn Any + Send>, Box<dyn Fn(Box<dyn Any + Send>)>)>>,
    main_exec_callback_tx: flume::Sender<Box<dyn Any + Send>>,
    main_exec_callback_rx: flume::Receiver<Box<dyn Any + Send>>,
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
        }
    }
}
impl Future for ShaderLoader {
    type Output = Box<dyn Any + Send>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        cx.waker().wake_by_ref();

        let this = unsafe { self.get_unchecked_mut() };
        let engine = (*dupe(this).engine).try_write();

        if engine.is_err() {
            return Poll::Pending;
        }
        dbg!("gave up");
        let engine = engine.unwrap();
        let bytes = this.bytes.clone();

        this.thread_main_exec_tx
            .send((
                Box::new(0),
                Box::new(move |_x| {
                    let render = engine.get_resource_no_init::<RenderManagerResource>();

                    let module = render.device.create_shader_module(ShaderModuleDescriptor {
                        label: Some("Shader from ShaderLoader"),
                        source: wgpu::ShaderSource::Wgsl(
                            String::from_utf8(bytes.clone()).unwrap().as_str().into(),
                        ),
                    });
                }),
            ))
            .unwrap();

        /*let module = render
        .device
        .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("camera_bind_group_layout"),
        });*/

        Poll::Ready(Box::new(0))
        //Poll::Ready(Box::new(module))
    }
}

impl FileLoadable for ShaderLoader {
    fn set_bytes(&mut self, file: std::io::Result<Vec<u8>>) {
        self.bytes = file.unwrap();
    }

    fn set_engine(&mut self, engine: SendEngineRuntime) {
        self.engine.set(engine);
    }
    fn set_thread_main_exec(
        &mut self,
        tx: flume::Sender<(Box<dyn Any + Send>, Box<dyn Fn(Box<dyn Any + Send>)>)>,
    ) {
        self.thread_main_exec_tx.set(tx);
    }
}
