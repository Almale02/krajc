use std::ops::Range;

use wgpu::{Buffer, RenderPass, RenderPipeline};

use crate::engine_runtime::EngineRuntime;

use super::asset::AssetHandleUntype;

pub trait MaterialGeneric {
    fn render_pipeline(&mut self, engine: &mut EngineRuntime) -> &RenderPipeline; //
    fn vertex_buffer(&self, engine: &mut EngineRuntime) -> &'static Buffer; //
    fn index_buffer(&self, engine: &mut EngineRuntime) -> &'static Buffer; //
    fn instance_buffer(&self, engine: &mut EngineRuntime) -> &'static Buffer; //
    fn setup_bind_groups(&mut self, engine: &mut EngineRuntime);
    fn set_bind_groups<'a>(&'a self, pipeline: &mut RenderPass<'a>, engine: &mut EngineRuntime);
    fn get_index_range(&self) -> Range<u32>;
    fn get_instance_range(&self) -> Range<u32>;
    fn register_systems(&self, engine: &mut EngineRuntime);
    fn get_shader_asset_handle(&self, engine: &mut EngineRuntime) -> AssetHandleUntype;
}

// InstanceGroupCall
