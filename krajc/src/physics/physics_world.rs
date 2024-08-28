use std::collections::HashSet;

use bevy_ecs::entity::Entity;
use bimap::BiHashMap;
use krajc_macros::EngineResource;
use rapier3d::dynamics::IslandManager as RapierIslandManager;
use rapier3d::dynamics::RigidBodyHandle;
use rapier3d::dynamics::RigidBodySet as RapierRigidBodySet;
use rapier3d::{
    dynamics::{
        CCDSolver as RapierCCDSolver, ImpulseJointSet as RapierImpulseJointSet,
        MultibodyJointSet as RapierMultibodyJointSet,
    },
    geometry::{
        ColliderHandle, ColliderSet as RapierColliderSet, DefaultBroadPhase,
        NarrowPhase as RapierNarrowPhase,
    },
    pipeline::{PhysicsPipeline as RapierPhysicsPipeline, QueryPipeline as RapierQueryPipeline},
};

#[derive(EngineResource, Default)]
pub struct PhysicsMappings {
    pub rigidbody_entity: BiHashMap<RigidBodyHandle, Entity>,
    pub collider_entity: BiHashMap<ColliderHandle, Entity>,
    pub added_colliders: HashSet<Entity>,
    pub added_bodies: HashSet<Entity>,
}

#[derive(EngineResource, Default)]
pub struct RigidBodySet(pub RapierRigidBodySet);
#[derive(EngineResource, Default)]
pub struct ColliderSet(pub RapierColliderSet);
#[derive(EngineResource, Default)]
pub struct IslandManager(pub RapierIslandManager);
#[derive(EngineResource, Default)]
pub struct MultibodyJointSet(pub RapierMultibodyJointSet);
#[derive(EngineResource, Default)]
pub struct ImpulseJointSet(pub RapierImpulseJointSet);
#[derive(EngineResource, Default)]
pub struct PhysicsPipeline(pub RapierPhysicsPipeline);
#[derive(EngineResource, Default)]
pub struct CcdSolver(pub RapierCCDSolver);
#[derive(EngineResource, Default)]
pub struct QueryPipeline(pub RapierQueryPipeline);
#[derive(EngineResource, Default)]
pub struct BroadPhase(pub DefaultBroadPhase);
#[derive(EngineResource, Default)]
pub struct NarrowPhase(pub RapierNarrowPhase);
