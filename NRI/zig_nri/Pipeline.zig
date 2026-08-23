const nri_c = @import("nri.zig");
const nri = @import("root.zig");
const Pipeline = @This();
const api = nri.api;
const err = @import("error.zig");

pub const PipelineCache = struct {
    pipeline_cache: *nri_c.PipelineCache,
    interface: *const nri.Interfaces,
    // DestroyPipelineCache: ?*const fn (pipelineCache: ?*PipelineCache) callconv(.c) void = null,
    pub fn destroy(self: *PipelineCache) void {
        self.interface.core.DestroyPipelineCache.?(self.pipeline_cache);
    }

    // GetPipelineCacheData: ?*const fn (pipelineCache: ?*PipelineCache, dst: ?*anyopaque, size: [*c]u64) callconv(.c) Result = null,
    pub fn get_data_size(self: *PipelineCache) !usize {
        var size: usize = 0;
        try err.checkResultC(self.interface.core.GetPipelineCacheData.?(self.pipeline_cache, null, &size));
        return size;
    }
    /// Returns the written data size
    pub fn get_data(self: *PipelineCache, buff: []u8) !usize {
        var size = buff.len;
        try err.checkResultC(self.interface.core.GetPipelineCacheData.?(self.pipeline_cache, buff.ptr, &size));
        return size;
    }
};
pub const PipelineLayout = struct {
    pipeline_layout: *nri_c.PipelineLayout,
    interface: *const nri.Interfaces,

    // DestroyPipelineLayout: ?*const fn (pipelineLayout: ?*PipelineLayout) callconv(.c) void = null,
    pub fn destroy(self: *PipelineLayout) void {
        self.interface.core.DestroyPipelineLayout.?(self.pipeline_layout);
    }
};
pipeline: *nri_c.Pipeline,
interface: *const nri.Interfaces,
// DestroyPipeline: ?*const fn (pipeline: ?*Pipeline) callconv(.c) void = null,
pub fn destroy(self: *Pipeline) void {
    self.interface.core.DestroyPipeline.?(self.pipeline);
}

test {
    @import("std").testing.refAllDecls(@This());
    @import("std").testing.refAllDecls(PipelineCache);
    @import("std").testing.refAllDecls(PipelineLayout);
}
