use cgmath::{Quaternion, Zero};

use crate::Pos;

use super::render_entity::RenderEntity;

pub struct TestInstanceSchemes;

impl TestInstanceSchemes {
    pub fn row(len: i32) -> Vec<RenderEntity> {
        let mut entity_list = vec![RenderEntity::default()];
        for i in 0..len {
            let pos = Pos::new(i as f32 * 1., 0., 0.);

            let rotation = Quaternion::<f32>::zero();
            let entity = RenderEntity::new(pos, rotation);

            entity_list.push(entity);
        }

        entity_list
    }
}

mod render_instance_macros {
    #[macro_export]
    macro_rules! create_instance_buffer {
        ( $instance_data:expr, $device:expr) => {
            $device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Instance Buffer"),
                contents: bytemuck::cast_slice(&$instance_data),
                usage: wgpu::BufferUsages::VERTEX,
            })
        };
    }
}
