use std::{
    any::Any,
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use tobj::LoadOptions;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BufferUsages,
};

use crate::{
    rendering::{
        asset::FinalAsset,
        managers::RenderManagerResource,
        mesh::mesh::{Mesh, TextureVertex},
    },
    typed_addr::dupe,
    Lateinit,
};

use super::file_resource_loader::{FileLoadable, SendEngineRuntime};

fn load_obj(mut reader: &[u8]) -> Result<(Vec<TextureVertex>, Vec<u16>), tobj::LoadError> {
    let load_result = tobj::load_obj_buf(
        &mut reader,
        &LoadOptions {
            triangulate: true,
            single_index: true,
            ..Default::default()
        },
        |path| tobj::load_mtl(path),
    )?;

    let (models, _materials_map) = load_result;

    // Use the first model for this example
    let model = &models[0];
    let mesh = &model.mesh;

    // Directly copy the vertices and indices
    let vertices: Vec<TextureVertex> = mesh
        .positions
        .chunks_exact(3)
        .zip(mesh.normals.chunks_exact(3))
        .zip(mesh.texcoords.chunks_exact(2))
        .map(|((pos, normal), texcoords)| TextureVertex {
            pos: [pos[0], pos[1], pos[2]],
            normal: [normal[0], normal[1], normal[2]],
            uv: [texcoords[0], texcoords[1]],
        })
        .collect();

    let indices: Vec<u16> = mesh.indices.iter().map(|&i| i as u16).collect();

    // Create buffers
    Ok((vertices, indices))
}

enum ObjLoaderState {
    Parsing,
    WaitingForMain,
}

pub struct ObjAsset {
    bytes: Vec<u8>,
    engine: Lateinit<SendEngineRuntime>,
    thread_main_exec_tx: Lateinit<flume::Sender<Box<dyn Fn()>>>,
    main_exec_callback_tx: flume::Sender<Box<dyn Any + Send>>,
    main_exec_callback_rx: flume::Receiver<Box<dyn Any + Send>>,
    state: ObjLoaderState,
}

impl Default for ObjAsset {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}

impl ObjAsset {
    pub fn new(bytes: Vec<u8>) -> Self {
        let (main_exec_callback_tx, main_exec_callback_rx) = flume::unbounded();
        Self {
            bytes,
            engine: Lateinit::default(),
            thread_main_exec_tx: Lateinit::default(),
            main_exec_callback_tx,
            main_exec_callback_rx,
            state: ObjLoaderState::Parsing,
        }
    }
}

impl FileLoadable for ObjAsset {
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

impl FinalAsset for ObjAsset {
    type FinalAsset = Mesh<TextureVertex>;
}

impl Future for ObjAsset {
    type Output = Box<dyn Any + Send>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = unsafe { self.get_unchecked_mut() };

        let engine = (*dupe(this).engine).try_write();

        if engine.is_err() {
            return Poll::Pending;
        }
        let mut engine = engine.unwrap();

        match &this.state {
            ObjLoaderState::Parsing => {
                // Parse OBJ data
                match load_obj(&this.bytes.as_slice()) {
                    Ok(obj) => {
                        let obj = obj.clone();
                        let tx = this.main_exec_callback_tx.clone();
                        let render = engine.get_resource::<RenderManagerResource>();
                        this.thread_main_exec_tx
                            .clone()
                            .send(Box::new(move || {
                                let (vertex, index) = obj.clone();

                                let vertex = vertex.into_boxed_slice();
                                let index = index.into_boxed_slice();

                                let vertex_buffer =
                                    render.device.create_buffer_init(&BufferInitDescriptor {
                                        label: Some("Vertex buffer from Obj model"),
                                        contents: bytemuck::cast_slice(&*vertex),
                                        usage: BufferUsages::VERTEX,
                                    });
                                //
                                let index_buffer =
                                    render.device.create_buffer_init(&BufferInitDescriptor {
                                        label: Some("Index buffer from Obj model"),
                                        contents: bytemuck::cast_slice(&*index),
                                        usage: BufferUsages::INDEX,
                                    });

                                let mesh = Mesh::<TextureVertex> {
                                    vertex_list: vertex,
                                    index_list: index,
                                    vertex_buffer,
                                    index_buffer,
                                };

                                tx.send(Box::new(mesh)).unwrap();
                            }))
                            .unwrap();
                        this.state = ObjLoaderState::WaitingForMain;
                        Poll::Pending
                    }
                    Err(e) => {
                        eprintln!("Failed to parse OBJ file: {}", e);
                        Poll::Ready(Box::new(0))
                    }
                }
            }
            ObjLoaderState::WaitingForMain => match this.main_exec_callback_rx.try_recv() {
                Ok(x) => Poll::Ready(x),
                Err(_) => {
                    cx.waker().wake_by_ref();
                    Poll::Pending
                }
            },
        }
    }
}
