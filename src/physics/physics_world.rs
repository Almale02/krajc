use bevy_ecs::entity::Entity;
use bimap::BiHashMap;
use cgmath::Vector2;
use rapier3d::{
    dynamics::{
        CCDSolver, ImpulseJointSet, IslandManager, MultibodyJointSet, RigidBody, RigidBodyHandle,
        RigidBodySet,
    },
    geometry::{BroadPhase, ColliderHandle, ColliderSet, DefaultBroadPhase, NarrowPhase},
    math::Real,
    na::Vector3,
    pipeline::{PhysicsPipeline, QueryPipeline},
};

use crate::{
    engine_runtime::schedule_manager::system_params::system_resource::EngineResource,
    typed_addr::dupe,
};

use super::Gravity;

type Type = PhysicsMappings;

pub struct PhysicsWorld {
    pub mappings: PhysicsMappings,

    pub rigid_body_set: RigidBodySet,
    pub collider_set: ColliderSet,
    pub multibody_joint_set: MultibodyJointSet,
    pub impulse_joint_set: ImpulseJointSet,
    pub physics_pipeline: PhysicsPipeline,
    pub island_manager: IslandManager,
    pub ccd_solver: CCDSolver,
    pub query_pipeline: QueryPipeline,
    pub broad_phase: DefaultBroadPhase,
    pub narrow_phase: NarrowPhase,
}
impl Default for PhysicsWorld {
    fn default() -> Self {
        Self {
            mappings: PhysicsMappings::default(),
            broad_phase: DefaultBroadPhase::new(),
            rigid_body_set: Default::default(),
            collider_set: Default::default(),
            multibody_joint_set: Default::default(),
            impulse_joint_set: Default::default(),
            physics_pipeline: Default::default(),
            island_manager: Default::default(),
            ccd_solver: Default::default(),
            query_pipeline: Default::default(),
            narrow_phase: Default::default(),
        }
    }
}
impl EngineResource for PhysicsWorld {
    fn get_mut(engine: &'static mut crate::engine_runtime::EngineRuntime) -> &'static mut Self {
        &mut engine.physics
    }
    fn get(engine: &'static mut crate::engine_runtime::EngineRuntime) -> &'static Self {
        &engine.physics
    }
}

#[derive(Default)]
pub struct PhysicsMappings {
    pub rigidbody_entity: BiHashMap<RigidBodyHandle, Entity>,
    pub collider_entity: BiHashMap<ColliderHandle, Entity>,
}

impl EngineResource for RigidBodySet {
    fn get_mut(engine: &'static mut crate::engine_runtime::EngineRuntime) -> &'static mut Self {
        &mut engine.physics.rigid_body_set
    }
    fn get(engine: &'static mut crate::engine_runtime::EngineRuntime) -> &'static Self {
        &engine.physics.rigid_body_set
    }
}
impl EngineResource for ColliderSet {
    fn get_mut(engine: &'static mut crate::engine_runtime::EngineRuntime) -> &'static mut Self {
        &mut engine.physics.collider_set
    }
    fn get(engine: &'static mut crate::engine_runtime::EngineRuntime) -> &'static Self {
        &engine.physics.collider_set
    }
}
impl EngineResource for PhysicsMappings {
    fn get_mut(engine: &'static mut crate::engine_runtime::EngineRuntime) -> &'static mut Self {
        &mut engine.physics.mappings
    }
    fn get(engine: &'static mut crate::engine_runtime::EngineRuntime) -> &'static Self {
        &engine.physics.mappings
    }
}
