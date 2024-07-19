use rapier3d::{math::Real, na::Vector3};

use crate::engine_runtime::schedule_manager::system_params::system_resource::EngineResource;

pub mod components;
pub mod physics_world;
pub mod system_params;
pub mod systems;

pub struct Gravity(Vector3<Real>);

impl EngineResource for Gravity {
    fn init(_engine: &mut crate::engine_runtime::EngineRuntime) -> &'static mut Self {
        Box::leak(Box::new(Gravity(Vector3::zeros())))
    }
}
