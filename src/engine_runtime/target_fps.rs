use crate::engine_runtime::EngineResource;

#[derive(Default, krajc::EngineResource)]
pub struct TargetFps(pub f32);
