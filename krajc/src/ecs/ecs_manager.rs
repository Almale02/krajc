use bevy_ecs::world::World;

pub struct EcsManager {
    pub world: World,
}
impl Default for EcsManager {
    fn default() -> Self {
        Self {
            world: World::new(),
        }
    }
}
