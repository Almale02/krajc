use std::{
    future::Future,
    io::BufRead,
    pin::Pin,
    task::{Context, Poll},
};

use mopa::Any;
use tobj::LoadOptions;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BufferUsages, Device,
};

use crate::{
    rendering::{
        asset::FinalAsset,
        mesh::mesh::{Mesh, TextureVertex},
    },
    Lateinit,
};

use super::file_resource_loader::{FileLoadable, SendEngineRuntime};

fn load_obj_to_mesh<B: BufRead>(
    device: &'static Device,
    mut reader: &[u8],
) -> Result<Mesh<TextureVertex>, Box<dyn std::error::Error>> {
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
    let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: Some("Vertex Buffer"),
        contents: bytemuck::cast_slice(&vertices),
        usage: BufferUsages::VERTEX,
    });
    let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: Some("Index Buffer"),
        contents: bytemuck::cast_slice(&indices),
        usage: BufferUsages::INDEX,
    });

    Ok(Mesh {
        vertex_list: vertices.into_boxed_slice(),
        index_list: indices.into_boxed_slice(),
        vertex_buffer,
        index_buffer,
    })
}

enum ObjLoaderState {
    Parsing,
    Ready,
}

pub struct ObjAsset {
    bytes: Vec<u8>,
    engine: Lateinit<SendEngineRuntime>,
    thread_main_exec_tx: Lateinit<flume::Sender<Box<dyn Fn()>>>,
    main_exec_callback_tx: flume::Sender<Box<dyn Any + Send>>,
    main_exec_callback_rx: flume::Receiver<Box<dyn Any + Send>>,
    state: ObjLoaderState,
    parsed_obj: Option<Mesh<TextureVertex>>,
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
            parsed_obj: None,
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

        match &this.state {
            ObjLoaderState::Parsing => {
                // Parse OBJ data
                match tobj::load_obj_buf(
                    &mut self.bytes.as_slice(),
                    &tobj::LoadOptions::default(),
                    |x| tobj::load_mtl(x),
                ) {
                    Ok(obj) => {
                        this.parsed_obj = Some(obj);
                        this.state = ObjLoaderState::Ready;
                        cx.waker().wake_by_ref();
                        Poll::Pending
                    }
                    Err(e) => {
                        eprintln!("Failed to parse OBJ file: {}", e);
                        this.state = ObjLoaderState::Ready; // Move to Ready state with no result
                        cx.waker().wake_by_ref();
                        Poll::Pending
                    }
                }
            }
            ObjLoaderState::Ready => {
                // Ready to return the result
                if let Some(parsed_obj) = &this.parsed_obj {
                    Poll::Ready(Box::new(parsed_obj.clone()) as Box<dyn Any + Send>)
                } else {
                    Poll::Ready(Box::new(Obj::default()) as Box<dyn Any + Send>)
                    // Empty OBJ
                }
            }
        }
    }
}
