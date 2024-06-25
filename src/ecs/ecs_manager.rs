use legion::{internals::iter::map::Into, World, WorldOptions};

pub struct EcsManager {
    pub world: World,
}
impl Default for EcsManager {
    fn default() -> Self {
        Self {
            world: World::new(WorldOptions {
                groups: Vec::default(),
            }),
        }
    }
}
