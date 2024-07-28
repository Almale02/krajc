use std::ops::DerefMut;

use bevy_ecs::{
    entity::Entity,
    query::{Added, Changed, With},
};
use krajc::system_fn;
use rapier3d::dynamics::{IntegrationParameters, IslandManager, RigidBodySet, RigidBodyType};

use crate::{
    engine_runtime::{
        schedule_manager::{
            runtime_schedule::RuntimePostEndFrameMainSchedule,
            system_params::{
                system_query::{EcsWorld, SystemQuery},
                system_resource::{EngineResource, Res},
            },
        },
        EngineRuntime,
    },
    physics::{
        components::general::{
            AngularVelocity, FixedRigidBody, LinearVelocity, PhysicsDontSyncRotation,
            PhysicsSyncDirectBodyModifications, RigidBody as RB, RigidBodyHandle as RBHandle,
            TargetKinematicTransform, TargetKinematicVelocity,
        },
        physics_world::{PhysicsMappings, PhysicsWorld},
        systems::collider::collider_systems,
        Gravity,
    },
    rendering::systems::general::Transform,
    typed_addr::dupe,
};

#[system_fn(RuntimePostEndFrameMainSchedule)]
pub fn step_physics(physics: Res<PhysicsWorld>, gravity: Res<Gravity>) {
    let event_handler = ();

    dupe(&physics).physics_pipeline.step(
        &gravity.0,
        &IntegrationParameters::default(),
        &mut dupe(&physics).island_manager,
        &mut dupe(&physics).broad_phase,
        &mut dupe(&physics).narrow_phase,
        &mut dupe(&physics).rigid_body_set,
        &mut dupe(&physics).collider_set,
        &mut dupe(&physics).impulse_joint_set,
        &mut dupe(&physics).multibody_joint_set,
        &mut dupe(&physics).ccd_solver,
        Some(&mut dupe(&physics).query_pipeline),
        &event_handler,
        &event_handler,
    )
}

#[system_fn(RuntimePostEndFrameMainSchedule)]
pub fn handle_rigidbody_insert(
    mut added: SystemQuery<(Entity, &RB, &Transform), Added<RB>>,
    mut ecs: EcsWorld,
    mut rigid_body_set: Res<RigidBodySet>,
    mut mappings: Res<PhysicsMappings>,
) {
    for (entity, body, transform) in added.iter() {
        if mappings.added_bodies.contains(&entity) {
            continue;
        }
        mappings.added_bodies.insert(entity);

        let mut created = body.0.clone();

        created.set_position(transform.iso, false);

        let body_type = created.body_type();

        let handle = rigid_body_set.insert(created);

        ecs.entity_mut(entity).insert((
            RBHandle::new(handle, dupe(&*rigid_body_set)),
            AngularVelocity::default(),
            LinearVelocity::default(),
        ));

        ecs.entity_mut(entity).remove::<RB>();

        match body_type {
            RigidBodyType::Dynamic => {
                //
            }
            RigidBodyType::Fixed => {
                //
                ecs.entity_mut(entity).insert(FixedRigidBody);
            }
            RigidBodyType::KinematicPositionBased => {
                //
                ecs.entity_mut(entity)
                    .insert(TargetKinematicTransform::default());
            }
            RigidBodyType::KinematicVelocityBased => {
                //
                ecs.entity_mut(entity)
                    .insert(TargetKinematicVelocity::default());
            }
        }

        mappings
            .rigidbody_entity
            .insert_no_overwrite(handle, entity)
            .unwrap();
    }
}

#[system_fn(RuntimePostEndFrameMainSchedule)]
pub fn sync_physics_transform(
    mut transforms: SystemQuery<(&mut Transform, Option<&PhysicsDontSyncRotation>), With<RBHandle>>,
    island_manager: Res<IslandManager>,
    mappings: Res<PhysicsMappings>,
    rigidbody_set: Res<RigidBodySet>,
) {
    island_manager
        .active_dynamic_bodies()
        .iter()
        .chain(island_manager.active_kinematic_bodies())
        .for_each(|handle| {
            let entity = mappings.rigidbody_entity.get_by_left(handle).unwrap();
            let (mut trans, sync_rotation) = transforms.get_mut(*entity).unwrap();
            let body = rigidbody_set.get(*handle).unwrap();

            *trans = Transform::new(*body.position());

            trans.translation = body.position().translation;
            if sync_rotation.is_none() {
                trans.rotation = body.position().rotation;
            }
        });
}
#[system_fn(RuntimePostEndFrameMainSchedule)]
pub fn sync_fixed_bodies_to_rapier(
    mut rigidbody_set: Res<RigidBodySet>,
    mut fixed: SystemQuery<(&RBHandle, &Transform), (With<FixedRigidBody> /*Changed<Transform>*/,)>,
) {
    for (handle, trans) in fixed.iter() {
        let body = rigidbody_set.get_mut(**handle).unwrap();

        body.set_position(**trans, true);
    }
}

#[system_fn(RuntimePostEndFrameMainSchedule)]
pub fn sync_target_transform_kinematic_body(
    mut target_transform: SystemQuery<
        (&TargetKinematicTransform, &RBHandle),
        //Changed<TargetKinematicTransform>,
    >,
    mut bodies: Res<RigidBodySet>,
) {
    for (target, handle) in target_transform.iter() {
        dbg!(*target.0);
        bodies
            .get_mut(handle.0)
            .unwrap()
            .set_next_kinematic_position(*target.0)
    }
}
#[system_fn(RuntimePostEndFrameMainSchedule)]
pub fn sync_target_vel_kinematic_body(
    mut target_transform: SystemQuery<
        (&TargetKinematicVelocity, &RBHandle),
        Changed<TargetKinematicVelocity>,
    >,
    mut bodies: Res<RigidBodySet>,
) {
    for (target, handle) in target_transform.iter() {
        bodies
            .get_mut(handle.0)
            .unwrap()
            .set_linvel(target.lin_vel, true);
        bodies
            .get_mut(handle.0)
            .unwrap()
            .set_angvel(target.ang_vel, true);
    }
}

#[system_fn(RuntimePostEndFrameMainSchedule)]
pub fn sync_physics_direct_transform_modification(
    mut transforms: SystemQuery<
        (&Transform, &RBHandle),
        (Changed<Transform>, With<PhysicsSyncDirectBodyModifications>),
    >,
    mut rigidbody_set: Res<RigidBodySet>,
) {
    for (transform, handle) in transforms.iter() {
        let body = rigidbody_set.get_mut(handle.0).unwrap();

        body.set_position(transform.iso, true)
    }
}

#[system_fn(RuntimePostEndFrameMainSchedule)]
pub fn mark_static_bodies_trans_changed(
    mut query: SystemQuery<&mut Transform, With<FixedRigidBody>>,
) {
    for mut trans in query.iter_mut() {
        let derefed = trans.deref_mut();
    }
}

#[system_fn(RuntimePostEndFrameMainSchedule)]
pub fn sync_ang_vel_to_physics(
    mut ang_vels: SystemQuery<(&RBHandle, &AngularVelocity), Changed<AngularVelocity>>,
    mut rigidbody_set: Res<RigidBodySet>,
) {
    for (handle, ang_vel) in ang_vels.iter() {
        let body = rigidbody_set.get_mut(handle.0).unwrap();

        body.set_angvel(ang_vel.0, true)
    }
}

#[system_fn(RuntimePostEndFrameMainSchedule)]
pub fn sync_lin_vel_to_physics(
    mut lin_vels: SystemQuery<(&RBHandle, &LinearVelocity), Changed<LinearVelocity>>,
    mut rigidbody_set: Res<RigidBodySet>,
) {
    for (handle, lin_vel) in lin_vels.iter() {
        let body = rigidbody_set.get_mut(handle.0).unwrap();

        body.set_linvel(lin_vel.0, true);
    }
}

pub fn physics_systems(runtime: &mut EngineRuntime) {
    sync_physics_transform!(runtime);
    sync_fixed_bodies_to_rapier!(runtime);
    handle_rigidbody_insert!(runtime);
    sync_target_transform_kinematic_body!(runtime);
    sync_target_vel_kinematic_body!(runtime);
    sync_physics_direct_transform_modification!(runtime);

    sync_ang_vel_to_physics!(runtime);
    sync_lin_vel_to_physics!(runtime);

    collider_systems(runtime);

    mark_static_bodies_trans_changed!(runtime);
}

/*impl EngineResource for TestRes {
    fn init(engine: &mut EngineRuntime) -> &'static mut Self {
        TypedAddr::new({
            let op = engine.static_resource_map.get_mut(&TypeId::of::<Self>());
            match op {
                Some(val) => *val,
                None => {
                    let new = Box::leak(Box::new(TestRes { test_field: 0 }));
                    let addr = TypedAddr::new_with_ref(new);
                    addr.addr
                }
            }
        })
        .get()
    }
}*/
