use cgmath::Vector2;
use rapier3d::{
    dynamics::{CCDSolver, ImpulseJointSet, IslandManager, MultibodyJointSet, RigidBodySet},
    geometry::{BroadPhase, ColliderSet, DefaultBroadPhase, NarrowPhase},
    pipeline::{PhysicsPipeline, QueryPipeline},
};

pub struct PhysicsWorld {
    pub rigid_body_set: RigidBodySet,
    pub collider_set: ColliderSet,
    pub multibody_joint_set: MultibodyJointSet,
    pub impulse_joint_set: ImpulseJointSet,
    pub physics_pipeline: PhysicsPipeline,
    pub island_manager: IslandManager,
    pub ccd_solver: CCDSolver,
    pub query_pipeline: QueryPipeline,
    pub broad_phase: Box<dyn BroadPhase>,
    pub narrow_phase: NarrowPhase,
}
impl Default for PhysicsWorld {
    fn default() -> Self {
        Self {
            broad_phase: Box::new(DefaultBroadPhase::new()),
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
