use bevy_ecs::component::Component;
use cgmath::Vector3;
use rapier3d::dynamics::{self as rapier};

use crate::rendering::systems::general::Transform;

#[derive(Component)]
pub enum RigidBody {
    Dynamic,
    Fixed,
    KinematicPositionBased,
    KinematicVelocityBased,
}
#[derive(Component)]
pub struct FixedRigidBody;
#[derive(Component)]
pub struct RigidBodyHandle(pub rapier::RigidBodyHandle);

impl std::ops::Deref for RigidBodyHandle {
    type Target = rapier::RigidBodyHandle;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Component, Default)]
pub struct TargetKinematicTransform(pub Transform);

pub struct TargetKinematicVelocity(pub LinVel, pub AngVel);

/// movement velocity
type LinVel = Vector3<f32>;

/// rotational velocity
type AngVel = Vector3<f32>;
