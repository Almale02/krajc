const std = @import("std");
const nri_c = @import("nri.zig");
const nri = @import("root.zig");
const Queue = @This();
const api = nri.api;
const err = @import("error.zig");

queue: *nri_c.Queue,
interface: *nri.Interfaces,

// CreateCommandAllocator: ?*const fn (queue: ?*Queue, commandAllocator: [*c]?*CommandAllocator) callconv(.c) Result = null,
pub fn createCommandAllocator(self: *Queue) !nri.CommandBuffer.CommandAllocator {
    var out_allocator: ?*nri_c.CommandAllocator = null;
    try err.checkResultC(self.interface.core.CreateCommandAllocator.?(self.queue, &out_allocator));
    return .{ .command_allocator = out_allocator.?, .interface = self.interface };
}

// QueueBeginAnnotation: ?*const fn (queue: ?*Queue, name: [*c]const u8, bgra: u32) callconv(.c) void = null,
pub fn beginAnnotation(self: *Queue, name: [:0]const u8, bgra: u32) void {
    self.interface.core.QueueBeginAnnotation.?(self.queue, name.ptr, bgra);
}

// QueueEndAnnotation: ?*const fn (queue: ?*Queue) callconv(.c) void = null,
pub fn endAnnotation(self: *Queue) void {
    self.interface.core.QueueEndAnnotation.?(self.queue);
}

// QueueAnnotation: ?*const fn (queue: ?*Queue, name: [*c]const u8, bgra: u32) callconv(.c) void = null,
pub fn annotation(self: *Queue, name: [:0]const u8, bgra: u32) void {
    self.interface.core.QueueAnnotation.?(self.queue, name.ptr, bgra);
}

pub const Timestamps = struct { cpu: u64, gpu: u64 };
// GetCalibratedTimestamps: ?*const fn (queue: ?*Queue, timestampGPU: [*c]u64, timestampCPU: [*c]u64) callconv(.c) void = null,
pub fn getCalibratedTimestamps(self: *Queue) Timestamps {
    var timestamps: Timestamps = undefined;
    self.interface.core.GetCalibratedTimestamps.?(self.queue, &timestamps.gpu, &timestamps.cpu);
    return timestamps;
}

// QueueSubmit: ?*const fn (queue: ?*Queue, queueSubmitDesc: [*c]const QueueSubmitDesc) callconv(.c) Result = null,
pub fn submit(self: *Queue, arena: std.mem.Allocator, desc: api.QueueSubmitDesc) !void {
    try err.checkResultC(self.interface.core.QueueSubmit.?(self.queue, &try desc.translateC(arena)));
}

// QueueWaitIdle: ?*const fn (queue: ?*Queue) callconv(.c) Result = null,
pub fn waitIdle(self: *Queue) !void {
    try err.checkResultC(self.interface.core.QueueWaitIdle.?(self.queue));
}

// GetQueueNativeObject: ?*const fn (queue: ?*const Queue) callconv(.c) ?*anyopaque = null,
pub fn getNativeObject(self: *const Queue) ?*anyopaque {
    return self.interface.core.GetQueueNativeObject.?(self.queue);
}
//// Helper
// UploadData: ?*const fn (queue: ?*Queue, textureUploadDescs: [*c]const TextureUploadDesc, textureUploadDescNum: u32, bufferUploadDescs: [*c]const BufferUploadDesc, bufferUploadDescNum: u32) callconv(.c) Result = null,
pub fn uploadData(self: *Queue, arena: std.mem.Allocator, texture_descs: []const api.TextureUploadDesc, buffer_descs: []const api.BufferUploadDesc) !void {
    const c_texture = try arena.alloc(nri_c.TextureUploadDesc, texture_descs.len);
    for (texture_descs, 0..) |vb, i| c_texture[i] = try vb.translateC(arena);
    const c_buffer = try arena.alloc(nri_c.BufferUploadDesc, texture_descs.len);
    for (buffer_descs, 0..) |vb, i| c_buffer[i] = vb.translateC();

    try err.checkResultC(self.interface.helper.UploadData.?(self.queue, c_texture.ptr, @intCast(texture_descs.len), c_buffer.ptr, @intCast(buffer_descs.len)));
}

test {
    @import("std").testing.refAllDecls(@This());
}
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
