use bevy_ecs::{entity::Entity, query::Added};
use krajc_macros::system_fn;
use rapier3d::{dynamics::RigidBodySet, geometry::ColliderSet};

use crate::{
    engine_runtime::{
        schedule_manager::{
            runtime_schedule::RuntimePhysicsSyncMainSchedule,
            schedule::IntoSystem,
            system_params::{system_query::SystemQuery, system_resource::Res},
        },
        EngineRuntime,
    },
    physics::{
        components::{collider::Collider, general::RigidBodyHandle},
        physics_world::PhysicsMappings,
    },
};

#[system_fn(RuntimePhysicsSyncMainSchedule)]
pub fn handle_collider_insert(
    mut inserted: SystemQuery<(Entity, &Collider, &RigidBodyHandle), Added<Collider>>,
    mut collider_set: Res<ColliderSet>,
    mut rigidbody_set: Res<RigidBodySet>,
    mut mappings: Res<PhysicsMappings>,
) {
    for (entity, coll, rigidbody_handle) in inserted.iter() {
        if mappings.added_colliders.contains(&entity) {
            continue;
        }
        //println!("Inserted collider for entity: {:?}", entity);
        mappings.added_colliders.insert(entity);

        let coll_handle =
            collider_set.insert_with_parent(coll.0.clone(), rigidbody_handle.0, &mut rigidbody_set);

        mappings
            .collider_entity
            .insert_no_overwrite(coll_handle, entity)
            .unwrap();
    }
}

pub fn collider_systems(runtime: &mut EngineRuntime) {
    runtime.register_system::<RuntimePhysicsSyncMainSchedule>(handle_collider_insert.system());
}
