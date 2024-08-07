use krajc::EngineResource;
use rapier3d::{math::Real, na::Vector3};

use crate::engine_runtime::schedule_manager::system_params::system_resource::EngineResource;

pub mod components;
pub mod physics_world;
pub mod system_params;
pub mod systems;

#[derive(Default, EngineResource)]
pub struct Gravity(pub Vector3<Real>);
