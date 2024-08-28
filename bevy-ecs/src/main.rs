use bevy_ecs::{component::Component, query::Changed, world::World};

#[deny(dead_code)]
#[derive(Component, Debug)]
struct Vel(u8);

fn main() {
    let mut world = World::new();
    let entities = world.spawn(Vel(0)).id();

    let mut query_changed = world.query_filtered::<&Vel, Changed<Vel>>();
    let mut query = world.query::<&mut Vel>();

    dbg!(query_changed.iter(&world).count());

    world.clear_trackers();
    query.get_mut(&mut world, entities).unwrap().0 += 1;
    dbg!(query_changed.iter(&world).count());
}
