use wgpu::{IndexFormat, RenderPass};

use crate::{engine_runtime::EngineRuntime, typed_addr::dupe};

use super::{asset::AssetHandleUntype, material::MaterialGeneric};

pub struct DrawPass {
    pub mat: Box<dyn MaterialGeneric>,
    pub res_handles: Vec<AssetHandleUntype>,
}
impl DrawPass {
    pub fn new(mat: Box<dyn MaterialGeneric>, res_handles: Vec<AssetHandleUntype>) -> Self {
        Self { mat, res_handles }
    }

    pub fn is_loaded(&self) -> bool {
        for x in self.res_handles.iter() {
            //dbg!(x.is_loaded());
            if !x.is_loaded() {
                return false;
            }
        }
        true
    }
    pub fn draw<'a>(&'a mut self, pass: &mut RenderPass<'a>, engine: &mut EngineRuntime) {
        pass.set_pipeline(dupe(self).mat.render_pipeline(engine));

        self.mat.setup_bind_groups(engine);
        self.mat.set_bind_groups(pass, engine);

        pass.set_vertex_buffer(0, self.mat.vertex_buffer(engine).slice(..));
        pass.set_index_buffer(self.mat.index_buffer(engine).slice(..), IndexFormat::Uint16);
        // set instance buffer
        pass.set_vertex_buffer(1, self.mat.instance_buffer(engine).slice(..));

        pass.draw_indexed(self.mat.get_index_range(), 0, self.mat.get_instance_range());
    }
}
