use bytemuck::{Pod, Zeroable};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Buffer, BufferUsages, Device,
};

pub trait Vertex {
    fn layout() -> wgpu::VertexBufferLayout<'static>;
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct TextureVertex {
    pub pos: [f32; 3],
    pub uv: [f32; 2],
    pub normal: [f32; 3],
}
impl TextureVertex {
    const ATTRIBS: [wgpu::VertexAttribute; 3] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2, 2 => Float32x3];
}
impl Vertex for TextureVertex {
    fn layout() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}
#[derive(Debug)]
pub struct Mesh<V: Vertex> {
    pub vertex_list: Box<[V]>,
    pub index_list: Box<[u16]>,
    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,
}

pub struct TextureVertexTemplates;
impl TextureVertexTemplates {
    pub fn cube(device: &Device) -> Mesh<TextureVertex> {
        #[rustfmt::skip]
        let vertex_list = Box::new([
            // Front face
            TextureVertex { pos: [-0.5, 0.5, 0.5], uv: [0.0, 1.0],normal: [0.0, 0.0, 1.0] }, // 0
            TextureVertex { pos: [-0.5, -0.5, 0.5], uv: [0.0, 0.0], normal: [0.0, 0.0, 1.0] }, // 1
            TextureVertex { pos: [0.5, -0.5, 0.5], uv: [1.0, 0.0], normal: [0.0, 0.0, 1.0] }, // 2
            TextureVertex { pos: [0.5, 0.5, 0.5], uv: [1.0, 1.0], normal: [0.0, 0.0, 1.0] }, // 3

            // Back face
            TextureVertex { pos: [0.5, 0.5, -0.5], uv: [0.0, 1.0],normal: [0.0, 0.0, -1.0] }, // 4
            TextureVertex { pos: [0.5, -0.5, -0.5], uv: [0.0, 0.0],normal: [0.0, 0.0, -1.0] }, // 5
            TextureVertex { pos: [-0.5, -0.5, -0.5], uv: [1.0, 0.0],normal: [0.0, 0.0, -1.0] }, // 6
            TextureVertex { pos: [-0.5, 0.5, -0.5], uv: [1.0, 1.0],normal: [0.0, 0.0, -1.0] }, // 7

            // Top face
            TextureVertex { pos: [-0.5, 0.5, -0.5], uv: [0.0, 1.0], normal: [0.0, 1.0, 0.0]}, // 8
            TextureVertex { pos: [-0.5, 0.5, 0.5], uv: [0.0, 0.0], normal: [0.0, 1.0, 0.0]}, // 9
            TextureVertex { pos: [0.5, 0.5, 0.5], uv: [1.0, 0.0], normal: [0.0, 1.0, 0.0]}, // 10
            TextureVertex { pos: [0.5, 0.5, -0.5], uv: [1.0, 1.0], normal: [0.0, 1.0, 0.0]}, // 11

            // Bottom face
            TextureVertex { pos: [-0.5, -0.5, 0.5], uv: [0.0, 1.0], normal: [0.0, -1.0, 0.0]}, // 12
            TextureVertex { pos: [-0.5, -0.5, -0.5], uv: [0.0, 0.0], normal: [0.0, -1.0, 0.0]}, // 13
            TextureVertex { pos: [0.5, -0.5, -0.5], uv: [1.0, 0.0], normal: [0.0, -1.0, 0.0]}, // 14
            TextureVertex { pos: [0.5, -0.5, 0.5], uv: [1.0, 1.0], normal: [0.0, -1.0, 0.0]}, // 15

            // Right face
            TextureVertex { pos: [0.5, 0.5, 0.5], uv: [0.0, 1.0], normal: [1.0, 0.0, 0.0]}, // 16
            TextureVertex { pos: [0.5, -0.5, 0.5], uv: [0.0, 0.0], normal: [1.0, 0.0, 0.0]}, // 17
            TextureVertex { pos: [0.5, -0.5, -0.5], uv: [1.0, 0.0], normal: [1.0, 0.0, 0.0]}, // 18
            TextureVertex { pos: [0.5, 0.5, -0.5], uv: [1.0, 1.0], normal: [1.0, 0.0, 0.0]}, // 19

            // Left face
            TextureVertex { pos: [-0.5, 0.5, -0.5], uv: [0.0, 1.0], normal: [-1.0, 0.0, 0.0]}, // 20
            TextureVertex { pos: [-0.5, -0.5, -0.5], uv: [0.0, 0.0], normal: [-1.0, 0.0, 0.0]}, // 21
            TextureVertex { pos: [-0.5, -0.5, 0.5], uv: [1.0, 0.0], normal: [-1.0, 0.0, 0.0]}, // 22
            TextureVertex { pos: [-0.5, 0.5, 0.5], uv: [1.0, 1.0], normal: [-1.0, 0.0, 0.0]}, // 23
        ]);

        #[rustfmt::skip]
        let index_list = Box::new([
            // Front face
            0, 1, 2, 0, 2, 3,
            // Back face
            4, 5, 6, 4, 6, 7,
            // Top face
            8, 9, 10, 8, 10, 11,
            // Bottom face
            12, 13, 14, 12, 14, 15,
            // Right face
            16, 17, 18, 16, 18, 19,
            // Left face
            20, 21, 22, 20, 22, 23,
        ]);

        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&*vertex_list),
            usage: BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&*index_list),
            usage: BufferUsages::INDEX,
        });
        Mesh {
            vertex_list,
            index_list,
            vertex_buffer,
            index_buffer,
        }
    }

    pub fn build_cube(device: &Device, width: f32, height: f32, depth: f32) -> Mesh<TextureVertex> {
        let half_width = width / 2.0;
        let half_height = height / 2.0;
        let half_depth = depth / 2.0;

        let vertex_list = Box::new([
            // Front face
            TextureVertex {
                pos: [-half_width, half_height, half_depth],
                uv: [0.0, 1.0],
                normal: [0.0, 0.0, 1.0],
            }, // 0
            TextureVertex {
                pos: [-half_width, -half_height, half_depth],
                uv: [0.0, 0.0],
                normal: [0.0, 0.0, 1.0],
            }, // 1
            TextureVertex {
                pos: [half_width, -half_height, half_depth],
                uv: [1.0, 0.0],
                normal: [0.0, 0.0, 1.0],
            }, // 2
            TextureVertex {
                pos: [half_width, half_height, half_depth],
                uv: [1.0, 1.0],
                normal: [0.0, 0.0, 1.0],
            }, // 3
            // Back face
            TextureVertex {
                pos: [half_width, half_height, -half_depth],
                uv: [0.0, 1.0],
                normal: [0.0, 0.0, -1.0],
            }, // 4
            TextureVertex {
                pos: [half_width, -half_height, -half_depth],
                uv: [0.0, 0.0],
                normal: [0.0, 0.0, -1.0],
            }, // 5
            TextureVertex {
                pos: [-half_width, -half_height, -half_depth],
                uv: [1.0, 0.0],
                normal: [0.0, 0.0, -1.0],
            }, // 6
            TextureVertex {
                pos: [-half_width, half_height, -half_depth],
                uv: [1.0, 1.0],
                normal: [0.0, 0.0, -1.0],
            }, // 7
            // Top face
            TextureVertex {
                pos: [-half_width, half_height, -half_depth],
                uv: [0.0, 1.0],
                normal: [0.0, 1.0, 0.0],
            }, // 8
            TextureVertex {
                pos: [-half_width, half_height, half_depth],
                uv: [0.0, 0.0],
                normal: [0.0, 1.0, 0.0],
            }, // 9
            TextureVertex {
                pos: [half_width, half_height, half_depth],
                uv: [1.0, 0.0],
                normal: [0.0, 1.0, 0.0],
            }, // 10
            TextureVertex {
                pos: [half_width, half_height, -half_depth],
                uv: [1.0, 1.0],
                normal: [0.0, 1.0, 0.0],
            }, // 11
            // Bottom face
            TextureVertex {
                pos: [-half_width, -half_height, half_depth],
                uv: [0.0, 1.0],
                normal: [0.0, -1.0, 0.0],
            }, // 12
            TextureVertex {
                pos: [-half_width, -half_height, -half_depth],
                uv: [0.0, 0.0],
                normal: [0.0, -1.0, 0.0],
            }, // 13
            TextureVertex {
                pos: [half_width, -half_height, -half_depth],
                uv: [1.0, 0.0],
                normal: [0.0, -1.0, 0.0],
            }, // 14
            TextureVertex {
                pos: [half_width, -half_height, half_depth],
                uv: [1.0, 1.0],
                normal: [0.0, -1.0, 0.0],
            }, // 15
            // Right face
            TextureVertex {
                pos: [half_width, half_height, half_depth],
                uv: [0.0, 1.0],
                normal: [1.0, 0.0, 0.0],
            }, // 16
            TextureVertex {
                pos: [half_width, -half_height, half_depth],
                uv: [0.0, 0.0],
                normal: [1.0, 0.0, 0.0],
            }, // 17
            TextureVertex {
                pos: [half_width, -half_height, -half_depth],
                uv: [1.0, 0.0],
                normal: [1.0, 0.0, 0.0],
            }, // 18
            TextureVertex {
                pos: [half_width, half_height, -half_depth],
                uv: [1.0, 1.0],
                normal: [1.0, 0.0, 0.0],
            }, // 19
            // Left face
            TextureVertex {
                pos: [-half_width, half_height, -half_depth],
                uv: [0.0, 1.0],
                normal: [-1.0, 0.0, 0.0],
            }, // 20
            TextureVertex {
                pos: [-half_width, -half_height, -half_depth],
                uv: [0.0, 0.0],
                normal: [-1.0, 0.0, 0.0],
            }, // 21
            TextureVertex {
                pos: [-half_width, -half_height, half_depth],
                uv: [1.0, 0.0],
                normal: [-1.0, 0.0, 0.0],
            }, // 22
            TextureVertex {
                pos: [-half_width, half_height, half_depth],
                uv: [1.0, 1.0],
                normal: [-1.0, 0.0, 0.0],
            }, // 23
        ]);

        #[rustfmt::skip]
        let index_list = Box::new([
            // Front face
            0, 1, 2, 0, 2, 3, // Back face
            4, 5, 6, 4, 6, 7, // Top face
            8, 9, 10, 8, 10, 11, // Bottom face
            12, 13, 14, 12, 14, 15, // Right face
            16, 17, 18, 16, 18, 19, // Left face
            20, 21, 22, 20, 22, 23,
        ]);

        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&*vertex_list),
            usage: BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&*index_list),
            usage: BufferUsages::INDEX,
        });
        Mesh {
            vertex_list,
            index_list,
            vertex_buffer,
            index_buffer,
        }
    }
}

#[macro_export]
macro_rules! create_vertex_buffer {
    ($device:expr, $vertex:expr) => {
        $device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&*$vertex),
            usage: BufferUsages::VERTEX,
        })
    };
}
