use std::{any::TypeId, ops::Deref};

use bevy_ecs::{
    entity::Entity,
    observer::TriggerTargets,
    query::{Added, Changed, With},
    system::IntoSystem,
};
use krajc::{system_fn, EngineResource};
use rapier3d::{
    data::Index,
    dynamics::{IntegrationParameters, RigidBody, RigidBodyBuilder, RigidBodyHandle, RigidBodySet},
    math::Isometry,
    na::Translation3,
    parry::simba::scalar::SupersetOf,
};
use syn::token::Typeof;

use crate::{
    engine_runtime::{
        schedule_manager::{
            runtime_schedule::{
                RuntimeEndFrameSchedule, RuntimeEngineLoadSchedule, RuntimeUpdateSchedule,
            },
            system_params::{
                system_query::{EcsWorld, SystemQuery},
                system_resource::{EngineResource, Res},
            },
        },
        EngineRuntime,
    },
    fps_logger, move_stuff_up,
    rendering::{buffer_manager::dupe, systems::general::Transform},
    startup,
    typed_addr::TypedAddr,
    update_rendering,
};

use super::{
    components::general::{
        self as comps, FixedRigidBody, RigidBody as RB, RigidBodyHandle as RBHandle,
        TargetKinematicTransform,
    },
    physics_world::{PhysicsMappings, PhysicsWorld},
    Gravity,
};

#[system_fn(RuntimeEndFrameSchedule)]
pub fn step_physics(mut physics: Res<PhysicsWorld>, gravity: Res<Gravity>) {
    let handle: RigidBodyHandle = physics
        .rigid_body_set
        .insert(RigidBodyBuilder::dynamic().position(Isometry::translation(5., 0., 0.)));

    let index: Index = handle.0;

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

#[system_fn(RuntimeUpdateSchedule)]
pub fn handle_rigidbody_insert(
    mut added: SystemQuery<(Entity, &RB), Added<RB>>,
    mut ecs: EcsWorld,
    mut rigid_body_set: Res<RigidBodySet>,
    mut mappings: Res<PhysicsMappings>,
) {
    for (entity, body) in added.iter() {
        let mut fixed = false;
        let mut kinematic_position = false;
        let mut kinematic_velocity = false;
        let created = match body {
            comps::RigidBody::Dynamic => RigidBodyBuilder::dynamic().build(),
            comps::RigidBody::Fixed => {
                fixed = true;
                RigidBodyBuilder::fixed().build()
            }
            comps::RigidBody::KinematicPositionBased => {
                kinematic_position = true;
                RigidBodyBuilder::kinematic_position_based().build()
            }
            comps::RigidBody::KinematicVelocityBased => {
                kinematic_velocity = true;
                RigidBodyBuilder::kinematic_velocity_based().build()
            }
        };
        let handle = rigid_body_set.insert(created);
        ecs.entity_mut(entity)
            .insert(comps::RigidBodyHandle(handle));
        if fixed {
            ecs.entity_mut(entity).insert(FixedRigidBody);
        }
        if kinematic_position {
            ecs.entity_mut(entity)
                .insert(TargetKinematicTransform::default());
        }

        mappings.rigidbody_entity.insert(handle, entity);
    }
}

#[system_fn(RuntimeUpdateSchedule)]
pub fn sync_physics_transform(
    mut transforms: SystemQuery<&mut Transform, With<RB>>,
    physics: Res<PhysicsWorld>,
) {
    physics
        .island_manager
        .active_dynamic_bodies()
        .iter()
        .chain(physics.island_manager.active_kinematic_bodies())
        .for_each(|handle| {
            let entity = *physics
                .mappings
                .rigidbody_entity
                .get_by_left(handle)
                .unwrap();
            let mut trans = transforms.get_mut(entity).unwrap();
            let body = physics.rigid_body_set.get(*handle).unwrap();
            *trans = Transform::new(*body.position());
        });
}
#[system_fn(RuntimeUpdateSchedule)]
pub fn sync_fixed_bodies_to_rapier(
    mut physics: Res<PhysicsWorld>,
    mut fixed: SystemQuery<(&RBHandle, &Transform), (With<FixedRigidBody>, Changed<Transform>)>,
) {
    for (handle, trans) in fixed.iter() {
        let body = physics.rigid_body_set.get_mut(**handle).unwrap();

        body.set_position(**trans, true);
    }
}

#[system_fn(RuntimeUpdateSchedule)]
pub fn syn_target_transform_kinematic_body(
    mut target_transform: SystemQuery<&TargetKinematicTransform, Changed<TargetKinematicTransform>>,
    mut physics: PhysicsWorld,
) {
}

pub fn physics_systems<'w>(runtime: &'w mut EngineRuntime<'w>) {
    sync_physics_transform!(runtime);
    sync_fixed_bodies_to_rapier!(runtime);
    handle_rigidbody_insert!(runtime);
    step_physics!(runtime);

    startup!(runtime);
    fps_logger!(runtime);
    update_rendering!(runtime);

    update_rendering!(runtime);

    //update_texture_material!(runtime);
    /*step_physics!(runtime);
    sync_physics_transform!(runtime);
    sync_fixed_bodies_to_rapier!(runtime);
    handle_rigidbody_insert!(runtime);*/

    move_stuff_up!(runtime);
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
