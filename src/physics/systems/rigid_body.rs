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
            runtime_schedule::RuntimePhysicsSyncMainSchedule,
            schedule::IntoSystem,
            system_params::{
                system_query::{EcsWorld, SystemQuery},
                system_resource::Res,
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
        Gravity,
    },
    rendering::systems::general::Transform,
    typed_addr::dupe,
};

#[system_fn(RuntimePhysicsSyncMainSchedule)]
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

#[system_fn(RuntimePhysicsSyncMainSchedule)]
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

#[system_fn(RuntimePhysicsSyncMainSchedule)]
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
#[system_fn(RuntimePhysicsSyncMainSchedule)]
pub fn sync_fixed_bodies_to_rapier(
    mut rigidbody_set: Res<RigidBodySet>,
    mut fixed: SystemQuery<(&RBHandle, &Transform), (With<FixedRigidBody> /*Changed<Transform>*/,)>,
) {
    for (handle, trans) in fixed.iter() {
        let body = rigidbody_set.get_mut(**handle).unwrap();

        body.set_position(**trans, true);
    }
}

#[system_fn(RuntimePhysicsSyncMainSchedule)]
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
#[system_fn(RuntimePhysicsSyncMainSchedule)]
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

#[system_fn(RuntimePhysicsSyncMainSchedule)]
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

#[system_fn(RuntimePhysicsSyncMainSchedule)]
fn mark_static_bodies_trans_changed(mut query: SystemQuery<&mut Transform, With<FixedRigidBody>>) {
    for mut trans in query.iter_mut() {
        let _ = trans.deref_mut();
    }
}

#[system_fn(RuntimePhysicsSyncMainSchedule)]
pub fn sync_ang_vel_to_physics(
    mut ang_vels: SystemQuery<(&RBHandle, &AngularVelocity), Changed<AngularVelocity>>,
    mut rigidbody_set: Res<RigidBodySet>,
) {
    for (handle, ang_vel) in ang_vels.iter() {
        let body = rigidbody_set.get_mut(handle.0).unwrap();

        body.set_angvel(ang_vel.0, true)
    }
}

#[system_fn(RuntimePhysicsSyncMainSchedule)]
pub fn sync_lin_vel_to_physics(
    mut lin_vels: SystemQuery<(&RBHandle, &LinearVelocity), Changed<LinearVelocity>>,
    mut rigidbody_set: Res<RigidBodySet>,
) {
    for (handle, lin_vel) in lin_vels.iter() {
        let body = rigidbody_set.get_mut(handle.0).unwrap();

        body.set_linvel(lin_vel.0, true);
    }
}

#[rustfmt::skip]
pub fn physics_systems(runtime: &mut EngineRuntime) {
    runtime.register_system::<RuntimePhysicsSyncMainSchedule>(sync_physics_transform.into_system());
    runtime.register_system::<RuntimePhysicsSyncMainSchedule>(sync_fixed_bodies_to_rapier.into_system());
    runtime.register_system::<RuntimePhysicsSyncMainSchedule>(handle_rigidbody_insert.into_system());
    runtime.register_system::<RuntimePhysicsSyncMainSchedule>(sync_target_transform_kinematic_body.into_system());
    runtime.register_system::<RuntimePhysicsSyncMainSchedule>(sync_target_vel_kinematic_body.into_system());
    runtime.register_system::<RuntimePhysicsSyncMainSchedule>(sync_physics_direct_transform_modification.into_system());

    runtime.register_system::<RuntimePhysicsSyncMainSchedule>(sync_ang_vel_to_physics.into_system());
    runtime.register_system::<RuntimePhysicsSyncMainSchedule>(sync_lin_vel_to_physics.into_system());

    runtime.register_system::<RuntimePhysicsSyncMainSchedule>(mark_static_bodies_trans_changed.into_system());

    //collider_systems(runtime);
}
