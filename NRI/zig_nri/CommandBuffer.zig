const std = @import("std");
const nri_c = @import("nri.zig");
const nri = @import("root.zig");
const CommandBuffer = @This();
const api = nri.api;
const err = @import("error.zig");

pub const CommandAllocator = struct {
    command_allocator: *nri_c.CommandAllocator,
    interface: *nri.Interfaces,
    // CreateCommandBuffer: ?*const fn (commandAllocator: ?*CommandAllocator, commandBuffer: [*c]?*CommandBuffer) callconv(.c) Result = null,
    pub fn createCommandBuffer(self: *CommandAllocator) !CommandBuffer {
        var out_buffer: ?*nri_c.CommandBuffer = null;
        try err.checkResultC(self.interface.core.CreateCommandBuffer.?(self.command_allocator, &out_buffer));
        return CommandBuffer{ .command_buffer = out_buffer.?, .interface = self.interface };
    }

    // DestroyCommandAllocator: ?*const fn (commandAllocator: ?*CommandAllocator) callconv(.c) void = null,
    pub fn destroy(self: *CommandAllocator) void {
        self.interface.core.DestroyCommandAllocator.?(self.command_allocator);
    }

    // ResetCommandAllocator: ?*const fn (commandAllocator: ?*CommandAllocator) callconv(.c) void = null,
    pub fn reset(self: *CommandAllocator) void {
        self.interface.core.ResetCommandAllocator.?(self.command_allocator);
    }
};

command_buffer: *nri_c.CommandBuffer,
interface: *const nri.Interfaces,

// DestroyCommandBuffer: ?*const fn (commandBuffer: ?*CommandBuffer) callconv(.c) void = null,
pub fn destroy(self: *CommandBuffer) void {
    self.interface.core.DestroyCommandBuffer.?(self.command_buffer);
}

// EndCommandBuffer: ?*const fn (commandBuffer: ?*CommandBuffer) callconv(.c) Result = null,
pub fn end(self: *CommandBuffer) !void {
    try err.checkResultC(self.interface.core.EndCommandBuffer.?(self.command_buffer));
}

// GetCommandBufferNativeObject: ?*const fn (commandBuffer: ?*const CommandBuffer) callconv(.c) ?*anyopaque = null,
pub fn getNativeObject(self: *const CommandBuffer) ?*anyopaque {
    return self.interface.core.GetCommandBufferNativeObject.?(self.command_buffer);
}

// BeginCommandBuffer: ?*const fn (commandBuffer: ?*CommandBuffer, descriptorPool: ?*const DescriptorPool) callconv(.c) Result = null,
pub fn begin(self: *CommandBuffer, descriptor_pool: *const nri.Descriptor.DescriptorPool) !void {
    try err.checkResultC(self.interface.core.BeginCommandBuffer.?(self.command_buffer, descriptor_pool.descriptor_pool));
}

// CmdSetDescriptorPool: ?*const fn (commandBuffer: ?*CommandBuffer, descriptorPool: ?*const DescriptorPool) callconv(.c) void = null,
pub fn setDescriptorPool(self: *CommandBuffer, descriptor_pool: *const nri.Descriptor.DescriptorPool) void {
    self.interface.core.CmdSetDescriptorPool.?(self.command_buffer, descriptor_pool.descriptor_pool);
} // ai here

// CmdSetPipelineLayout: ?*const fn (commandBuffer: ?*CommandBuffer, bindPoint: BindPoint, pipelineLayout: ?*const PipelineLayout) callconv(.c) void = null,
pub fn setPipelineLayout(self: *CommandBuffer, bind_point: api.BindPoint, pipeline_layout: *const nri.Pipeline.PipelineLayout) void {
    self.interface.core.CmdSetPipelineLayout.?(self.command_buffer, @intFromEnum(bind_point), pipeline_layout.pipeline_layout);
}

// CmdSetDescriptorSet: ?*const fn (commandBuffer: ?*CommandBuffer, setDescriptorSetDesc: [*c]const SetDescriptorSetDesc) callconv(.c) void = null,
pub fn setDescriptorSet(self: *CommandBuffer, desc: api.SetDescriptorSetDesc) void {
    self.interface.core.CmdSetDescriptorSet.?(self.command_buffer, &desc.translateC());
}

// CmdSetRootConstants: ?*const fn (commandBuffer: ?*CommandBuffer, setRootConstantsDesc: [*c]const SetRootConstantsDesc) callconv(.c) void = null,
pub fn setRootConstants(self: *CommandBuffer, desc: api.SetRootConstantsDesc) void {
    self.interface.core.CmdSetRootConstants.?(self.command_buffer, &desc.translateC());
}

// CmdSetRootDescriptor: ?*const fn (commandBuffer: ?*CommandBuffer, setRootDescriptorDesc: [*c]const SetRootDescriptorDesc) callconv(.c) void = null,
pub fn setRootDescriptor(self: *CommandBuffer, desc: api.SetRootDescriptorDesc) void {
    self.interface.core.CmdSetRootDescriptor.?(self.command_buffer, &desc.translateC());
}

// CmdSetPipeline: ?*const fn (commandBuffer: ?*CommandBuffer, pipeline: ?*const Pipeline) callconv(.c) void = null,
pub fn setPipeline(self: *CommandBuffer, pipeline: *const nri.Pipeline) void {
    self.interface.core.CmdSetPipeline.?(self.command_buffer, pipeline.pipeline);
}

// CmdBarrier: ?*const fn (commandBuffer: ?*CommandBuffer, barrierDesc: [*c]const BarrierDesc) callconv(.c) void = null,
pub fn barrier(self: *CommandBuffer, arena: std.mem.Allocator, desc: api.BarrierDesc) !void {
    self.interface.core.CmdBarrier.?(self.command_buffer, &try desc.translateC(arena));
}

// CmdSetIndexBuffer: ?*const fn (commandBuffer: ?*CommandBuffer, buffer: ?*const Buffer, offset: u64, indexType: IndexType) callconv(.c) void = null,
pub fn setIndexBuffer(self: *CommandBuffer, buffer: *const nri.Buffer, offset: u64, index_type: api.IndexType) void {
    self.interface.core.CmdSetIndexBuffer.?(self.command_buffer, buffer.buffer, offset, @intFromEnum(index_type));
}

// CmdSetVertexBuffers: ?*const fn (commandBuffer: ?*CommandBuffer, baseSlot: u32, vertexBufferDescs: [*c]const VertexBufferDesc, vertexBufferNum: u32) callconv(.c) void = null,
pub fn setVertexBuffers(self: *CommandBuffer, arena: std.mem.Allocator, base_slot: u32, vertex_buffer_descs: []const api.VertexBufferDesc) !void {
    const c_descs = try arena.alloc(nri_c.VertexBufferDesc, vertex_buffer_descs.len);
    for (vertex_buffer_descs, 0..) |vb, i| c_descs[i] = vb.translateC();
    self.interface.core.CmdSetVertexBuffers.?(self.command_buffer, base_slot, c_descs.ptr, @intCast(vertex_buffer_descs.len));
}

// CmdSetViewports: ?*const fn (commandBuffer: ?*CommandBuffer, viewports: [*c]const Viewport, viewportNum: u32) callconv(.c) void = null,
pub fn setViewports(self: *CommandBuffer, arena: std.mem.Allocator, viewports: []const api.Viewport) !void {
    const c_viewports = try arena.alloc(nri_c.Viewport, viewports.len);
    for (viewports, 0..) |vp, i| c_viewports[i] = vp.translateC();
    self.interface.core.CmdSetViewports.?(self.command_buffer, c_viewports.ptr, @intCast(viewports.len));
}

// CmdSetScissors: ?*const fn (commandBuffer: ?*CommandBuffer, rects: [*c]const Rect, rectNum: u32) callconv(.c) void = null,
pub fn setScissors(self: *CommandBuffer, arena: std.mem.Allocator, rects: []const api.Rect) !void {
    const c_rects = try arena.alloc(nri_c.Rect, rects.len);
    for (rects, 0..) |rc, i| c_rects[i] = rc.translateC();
    self.interface.core.CmdSetScissors.?(self.command_buffer, c_rects.ptr, @intCast(rects.len));
}

// CmdSetStencilReference: ?*const fn (commandBuffer: ?*CommandBuffer, frontRef: u8, backRef: u8) callconv(.c) void = null,
pub fn setStencilReference(self: *CommandBuffer, front_ref: u8, back_ref: u8) void {
    self.interface.core.CmdSetStencilReference.?(self.command_buffer, front_ref, back_ref);
}

// CmdSetDepthBounds: ?*const fn (commandBuffer: ?*CommandBuffer, boundsMin: f32, boundsMax: f32) callconv(.c) void = null,
pub fn setDepthBounds(self: *CommandBuffer, bounds_min: f32, bounds_max: f32) void {
    self.interface.core.CmdSetDepthBounds.?(self.command_buffer, bounds_min, bounds_max);
}

// CmdSetBlendConstants: ?*const fn (commandBuffer: ?*CommandBuffer, color: [*c]const Color32f) callconv(.c) void = null,
pub fn setBlendConstants(self: *CommandBuffer, color: api.Color32f) void {
    self.interface.core.CmdSetBlendConstants.?(self.command_buffer, &color.translateC());
}

// CmdSetSampleLocations: ?*const fn (commandBuffer: ?*CommandBuffer, locations: [*c]const SampleLocation, locationNum: Sample_t, sampleNum: Sample_t) callconv(.c) void = null,
pub fn setSampleLocations(self: *CommandBuffer, arena: std.mem.Allocator, locations: []const api.SampleLocation, sample_num: u8) !void {
    const c_locations = try arena.alloc(nri_c.SampleLocation, locations.len);
    for (locations, 0..) |loc, i| c_locations[i] = loc.translateC();
    self.interface.core.CmdSetSampleLocations.?(self.command_buffer, c_locations.ptr, @intCast(locations.len), sample_num);
}

// CmdSetShadingRate: ?*const fn (commandBuffer: ?*CommandBuffer, shadingRateDesc: [*c]const ShadingRateDesc) callconv(.c) void = null,
pub fn setShadingRate(self: *CommandBuffer, desc: api.ShadingRateDesc) void {
    self.interface.core.CmdSetShadingRate.?(self.command_buffer, &desc.translateC());
}

// CmdSetDepthBias: ?*const fn (commandBuffer: ?*CommandBuffer, depthBiasDesc: [*c]const DepthBiasDesc) callconv(.c) void = null,
pub fn setDepthBias(self: *CommandBuffer, desc: api.DepthBiasDesc) void {
    self.interface.core.CmdSetDepthBias.?(self.command_buffer, &desc.translateC());
}

// CmdBeginRendering: ?*const fn (commandBuffer: ?*CommandBuffer, renderingDesc: [*c]const RenderingDesc) callconv(.c) void = null,
pub fn beginRendering(self: *CommandBuffer, arena: std.mem.Allocator, desc: api.RenderingDesc) !void {
    self.interface.core.CmdBeginRendering.?(self.command_buffer, &try desc.translateC(arena));
}

// CmdClearAttachments: ?*const fn (commandBuffer: ?*CommandBuffer, clearAttachmentDescs: [*c]const ClearAttachmentDesc, clearAttachmentDescNum: u32, rects: [*c]const Rect, rectNum: u32) callconv(.c) void = null,
pub fn clearAttachments(self: *CommandBuffer, arena: std.mem.Allocator, clears: []const api.ClearAttachmentDesc, rects: []const api.Rect) !void {
    const c_clears = try arena.alloc(nri_c.ClearAttachmentDesc, clears.len);
    for (clears, 0..) |cl, i| c_clears[i] = cl.translateC();
    const c_rects = try arena.alloc(nri_c.Rect, rects.len);
    for (rects, 0..) |rc, i| c_rects[i] = rc.translateC();
    self.interface.core.CmdClearAttachments.?(self.command_buffer, c_clears.ptr, @intCast(clears.len), c_rects.ptr, @intCast(rects.len));
}
// CmdDraw: ?*const fn (commandBuffer: ?*CommandBuffer, drawDesc: [*c]const DrawDesc) callconv(.c) void = null,
pub fn draw(self: *CommandBuffer, draw_desc: api.DrawDesc) void {
    self.interface.core.CmdDraw.?(self.command_buffer, &draw_desc.translateC());
}

// CmdDrawIndexed: ?*const fn (commandBuffer: ?*CommandBuffer, drawIndexedDesc: [*c]const DrawIndexedDesc) callconv(.c) void = null,
pub fn drawIndexed(self: *CommandBuffer, draw_indexed_desc: api.DrawIndexedDesc) void {
    self.interface.core.CmdDrawIndexed.?(self.command_buffer, &draw_indexed_desc.translateC());
}

// CmdDrawIndirect: ?*const fn (commandBuffer: ?*CommandBuffer, buffer: ?*const Buffer, offset: u64, drawNum: u32, stride: u32, countBuffer: ?*const Buffer, countBufferOffset: u64) callconv(.c) void = null,
pub fn drawIndirect(self: *CommandBuffer, buffer: *const nri.Buffer, offset: u64, draw_num: u32, stride: u32, count_buffer: ?*const nri.Buffer, count_buffer_offset: u64) void {
    self.interface.core.CmdDrawIndirect.?(
        self.command_buffer,
        buffer.buffer,
        offset,
        draw_num,
        stride,
        if (count_buffer) |cb| cb.buffer else null,
        count_buffer_offset,
    );
}

// CmdDrawIndexedIndirect: ?*const fn (commandBuffer: ?*CommandBuffer, buffer: ?*const Buffer, offset: u64, drawNum: u32, stride: u32, countBuffer: ?*const Buffer, countBufferOffset: u64) callconv(.c) void = null,
pub fn drawIndexedIndirect(self: *CommandBuffer, buffer: *const nri.Buffer, offset: u64, draw_num: u32, stride: u32, count_buffer: ?*const nri.Buffer, count_buffer_offset: u64) void {
    self.interface.core.CmdDrawIndexedIndirect.?(
        self.command_buffer,
        buffer.buffer,
        offset,
        draw_num,
        stride,
        if (count_buffer) |cb| cb.buffer else null,
        count_buffer_offset,
    );
}

// CmdEndRendering: ?*const fn (commandBuffer: ?*CommandBuffer) callconv(.c) void = null,
pub fn endRendering(self: *CommandBuffer) void {
    self.interface.core.CmdEndRendering.?(self.command_buffer);
}

// CmdDispatch: ?*const fn (commandBuffer: ?*CommandBuffer, dispatchDesc: [*c]const DispatchDesc) callconv(.c) void = null,
pub fn dispatch(self: *CommandBuffer, dispatch_desc: api.DispatchDesc) void {
    self.interface.core.CmdDispatch.?(self.command_buffer, &dispatch_desc.translateC());
}

// CmdDispatchIndirect: ?*const fn (commandBuffer: ?*CommandBuffer, buffer: ?*const Buffer, offset: u64) callconv(.c) void = null,
pub fn dispatchIndirect(self: *CommandBuffer, buffer: *const nri.Buffer, offset: u64) void {
    self.interface.core.CmdDispatchIndirect.?(self.command_buffer, buffer.buffer, offset);
}

// CmdCopyBuffer: ?*const fn (commandBuffer: ?*CommandBuffer, dstBuffer: ?*Buffer, dstOffset: u64, srcBuffer: ?*const Buffer, srcOffset: u64, size: u64) callconv(.c) void = null,
pub fn copyBuffer(self: *CommandBuffer, dst_buffer: *nri.Buffer, dst_offset: u64, src_buffer: *const nri.Buffer, src_offset: u64, size: u64) void {
    self.interface.core.CmdCopyBuffer.?(self.command_buffer, dst_buffer.buffer, dst_offset, src_buffer.buffer, src_offset, size);
}

// CmdCopyTexture: ?*const fn (commandBuffer: ?*CommandBuffer, dstTexture: ?*Texture, dstRegion: [*c]const TextureRegionDesc, srcTexture: ?*const Texture, srcRegion: [*c]const TextureRegionDesc) callconv(.c) void = null,
pub fn copyTexture(self: *CommandBuffer, dst_texture: *nri.Texture, dst_region: api.TextureRegionDesc, src_texture: *const nri.Texture, src_region: api.TextureRegionDesc) void {
    self.interface.core.CmdCopyTexture.?(
        self.command_buffer,
        dst_texture.texture,
        &dst_region.translateC(),
        src_texture.texture,
        &src_region.translateC(),
    );
}

// CmdUploadBufferToTexture: ?*const fn (commandBuffer: ?*CommandBuffer, dstTexture: ?*Texture, dstRegion: [*c]const TextureRegionDesc, srcBuffer: ?*const Buffer, srcDataLayout: [*c]const TextureDataLayoutDesc) callconv(.c) void = null,
pub fn uploadBufferToTexture(self: *CommandBuffer, dst_texture: *nri.Texture, dst_region: api.TextureRegionDesc, src_buffer: *const nri.Buffer, src_data_layout: api.TextureDataLayoutDesc) void {
    self.interface.core.CmdUploadBufferToTexture.?(self.command_buffer, dst_texture.texture, &dst_region.translateC(), src_buffer.buffer, &src_data_layout.translateC());
}

// CmdReadbackTextureToBuffer: ?*const fn (commandBuffer: ?*CommandBuffer, dstBuffer: ?*Buffer, dstDataLayout: [*c]const TextureDataLayoutDesc, srcTexture: ?*const Texture, srcRegion: [*c]const TextureRegionDesc) callconv(.c) void = null,
pub fn readbackTextureToBuffer(self: *CommandBuffer, dst_buffer: *nri.Buffer, dst_data_layout: api.TextureDataLayoutDesc, src_texture: *const nri.Texture, src_region: api.TextureRegionDesc) void {
    self.interface.core.CmdReadbackTextureToBuffer.?(self.command_buffer, dst_buffer.buffer, &dst_data_layout.translateC(), src_texture.texture, &src_region.translateC());
}

// CmdZeroBuffer: ?*const fn (commandBuffer: ?*CommandBuffer, buffer: ?*Buffer, offset: u64, size: u64) callconv(.c) void = null,
pub fn zeroBuffer(self: *CommandBuffer, buffer: *nri.Buffer, offset: u64, size: u64) void {
    self.interface.core.CmdZeroBuffer.?(self.command_buffer, buffer.buffer, offset, size);
}

// CmdResolveTexture: ?*const fn (commandBuffer: ?*CommandBuffer, dstTexture: ?*Texture, dstRegion: [*c]const TextureRegionDesc, srcTexture: ?*const Texture, srcRegion: [*c]const TextureRegionDesc, resolveOp: ResolveOp) callconv(.c) void = null,
pub fn resolveTexture(self: *CommandBuffer, dst_texture: *nri.Texture, dst_region: api.TextureRegionDesc, src_texture: *const nri.Texture, src_region: api.TextureRegionDesc, resolve_op: api.ResolveOp) void {
    self.interface.core.CmdResolveTexture.?(
        self.command_buffer,
        dst_texture.texture,
        &dst_region.translateC(),
        src_texture.texture,
        &src_region.translateC(),
        @intFromEnum(resolve_op),
    );
}

// CmdClearStorage: ?*const fn (commandBuffer: ?*CommandBuffer, clearStorageDesc: [*c]const ClearStorageDesc) callconv(.c) void = null,
pub fn clearStorage(self: *CommandBuffer, clear_storage_desc: api.ClearStorageDesc) void {
    self.interface.core.CmdClearStorage.?(self.command_buffer, &clear_storage_desc.translateC());
}

// CmdResetQueries: ?*const fn (commandBuffer: ?*CommandBuffer, queryPool: ?*QueryPool, offset: u32, num: u32) callconv(.c) void = null,
pub fn resetQueries(self: *CommandBuffer, query_pool: *nri.QueryPool, offset: u32, num: u32) void {
    self.interface.core.CmdResetQueries.?(self.command_buffer, query_pool.query_pool, offset, num);
}

// CmdBeginQuery: ?*const fn (commandBuffer: ?*CommandBuffer, queryPool: ?*QueryPool, offset: u32) callconv(.c) void = null,
pub fn beginQuery(self: *CommandBuffer, query_pool: *nri.QueryPool, offset: u32) void {
    self.interface.core.CmdBeginQuery.?(self.command_buffer, query_pool.query_pool, offset);
}

// CmdEndQuery: ?*const fn (commandBuffer: ?*CommandBuffer, queryPool: ?*QueryPool, offset: u32) callconv(.c) void = null,
pub fn endQuery(self: *CommandBuffer, query_pool: *nri.QueryPool, offset: u32) void {
    self.interface.core.CmdEndQuery.?(self.command_buffer, query_pool.query_pool, offset);
}

// CmdCopyQueries: ?*const fn (commandBuffer: ?*CommandBuffer, queryPool: ?*const QueryPool, offset: u32, num: u32, dstBuffer: ?*Buffer, dstOffset: u64) callconv(.c) void = null,
pub fn copyQueries(self: *CommandBuffer, query_pool: *const nri.QueryPool, offset: u32, num: u32, dst_buffer: *nri.Buffer, dst_offset: u64) void {
    self.interface.core.CmdCopyQueries.?(self.command_buffer, query_pool.query_pool, offset, num, dst_buffer.buffer, dst_offset);
}

// CmdBeginAnnotation: ?*const fn (commandBuffer: ?*CommandBuffer, name: [*c]const u8, bgra: u32) callconv(.c) void = null,
pub fn beginAnnotation(self: *CommandBuffer, name: [:0]const u8, bgra: u32) void {
    self.interface.core.CmdBeginAnnotation.?(self.command_buffer, name.ptr, bgra);
}

// CmdEndAnnotation: ?*const fn (commandBuffer: ?*CommandBuffer) callconv(.c) void = null,
pub fn endAnnotation(self: *CommandBuffer) void {
    self.interface.core.CmdEndAnnotation.?(self.command_buffer);
}

// CmdAnnotation: ?*const fn (commandBuffer: ?*CommandBuffer, name: [*c]const u8, bgra: u32) callconv(.c) void = null,
pub fn annotation(self: *CommandBuffer, name: [:0]const u8, bgra: u32) void {
    self.interface.core.CmdAnnotation.?(self.command_buffer, name.ptr, bgra);
}

test {
    @import("std").testing.refAllDecls(@This());
    @import("std").testing.refAllDecls(CommandAllocator);
}
