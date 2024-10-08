use krajc_macros::EngineResource;
use rapier3d::{math::Real, na::Vector3};

pub mod components;
pub mod physics_world;
pub mod system_params;
pub mod systems;

#[derive(Default, krajc_macros::EngineResource)]
pub struct Gravity(pub Vector3<Real>);
