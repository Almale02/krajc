use crate::engine_runtime::EngineRuntime;
use crate::rendering::managers::RenderManagerResource;
use crate::typed_addr::dupe;
use crate::typed_addr::TypedAddr;
use crate::AssetLoader;
use crate::Lateinit;
use futures::future::BoxFuture;
use futures::task::Context;
use futures::task::Poll;
use futures::FutureExt;
use std::any::Any;
use std::future::Future;
use std::pin::Pin;
use tokio::fs;
use wgpu::ShaderModuleDescriptor;

pub struct FileResourceLoader<T: FileLoadable + Future<Output = Box<dyn Any + Send>> + 'static> {
    pub path: &'static str,
    file_loader: BoxFuture<'static, tokio::io::Result<Vec<u8>>>,
    loaded_file: bool,
    processor: T,
    engine: TypedAddr<EngineRuntime>,
}

impl<T: FileLoadable + 'static + Future<Output = Box<dyn Any + Send>> + Send + Unpin + Default>
    FileResourceLoader<T>
{
    pub fn new(path: &'static str) -> Self {
        Self {
            path,
            file_loader: fs::read(path).boxed(),
            loaded_file: false,
            processor: T::default(),
            engine: TypedAddr::default(),
        }
    }
}

impl<T: FileLoadable + Future<Output = Box<dyn Any + Send>> + Send + Unpin + 'static> AssetLoader
    for FileResourceLoader<T>
{
    fn set_engine(&mut self, engine: &'static mut crate::engine_runtime::EngineRuntime) {
        self.engine = TypedAddr::new_with_ref(engine);
    }
}

impl<T: FileLoadable + Future<Output = Box<dyn Any + Send>> + 'static + Send + Unpin> Future
    for FileResourceLoader<T>
{
    type Output = T::Output;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let engine = self.engine.get();
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
    fn set_engine(&mut self, engine: &'static mut EngineRuntime);
}

/// could be use with FileResourceLoader
/// returns Vec<u8> of the file
#[derive(Default)]
pub struct RawFileLoader {
    pub bytes: Vec<u8>,
    engine: Lateinit<&'static mut EngineRuntime>,
}
impl RawFileLoader {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self {
            bytes,
            engine: Lateinit::default(),
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

    fn set_engine(&mut self, engine: &'static mut EngineRuntime) {
        self.engine.set(engine);
    }
}

impl<T> Unpin for FileResourceLoader<T> where
    T: FileLoadable + Send + Future<Output = Box<(dyn Any + Send + 'static)>> + 'static
{
}

/// could be use with FileResourceLoader
/// returns the ShaderModule
#[derive(Default)]
pub struct ShaderLoader {
    pub bytes: Vec<u8>,
    engine: Lateinit<&'static mut EngineRuntime>,
}
impl ShaderLoader {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self {
            bytes,
            engine: Lateinit::default(),
        }
    }
}
impl Future for ShaderLoader {
    type Output = Box<dyn Any + Send>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        cx.waker().wake_by_ref();

        let render = self
            .as_mut()
            .engine
            .as_option_mut()
            .expect("engine at shader loader was none")
            .get_resource::<RenderManagerResource>();

        let module = render.device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Shader from ShaderLoader"),
            source: wgpu::ShaderSource::Wgsl(
                String::from_utf8(self.bytes.clone())
                    .unwrap()
                    .as_str()
                    .into(),
            ),
        });
        Poll::Ready(Box::new(module))
    }
}

impl FileLoadable for ShaderLoader {
    fn set_bytes(&mut self, file: std::io::Result<Vec<u8>>) {
        self.bytes = file.unwrap();
    }

    fn set_engine(&mut self, engine: &'static mut EngineRuntime) {
        self.engine.set(engine);
    }
}
