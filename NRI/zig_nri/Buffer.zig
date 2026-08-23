const std = @import("std");
const nri_c = @import("nri.zig");
const nri = @import("root.zig");
const Buffer = @This();
const api = nri.api;
const err = @import("error.zig");

buffer: *nri_c.Buffer,

// GetBufferDesc: ?*const fn (buffer: ?*const Buffer) callconv(.c) [*c]const BufferDesc = null,
pub fn getDesc(self: Buffer, device: *const nri.Device) api.BufferDesc {
    const desc = device.interface.core.GetBufferDesc.?(self.buffer);
    return nri.fromC(api.BufferDesc, desc.*);
}

// DestroyBuffer: ?*const fn (buffer: ?*Buffer) callconv(.c) void = null,
pub fn destroy(self: Buffer, device: *const nri.Device) void {
    device.interface.core.DestroyBuffer.?(self.buffer);
}

// GetBufferMemoryDesc: ?*const fn (buffer: ?*const Buffer, memoryLocation: MemoryLocation, memoryDesc: [*c]MemoryDesc) callconv(.c) void = null,
pub fn getMemoryDesc(self: Buffer, device: *const nri.Device, mem_loc: api.MemoryLocation) api.MemoryDesc {
    var out_desc: nri_c.MemoryDesc = undefined;
    device.interface.core.GetBufferMemoryDesc.?(self.buffer, @intFromEnum(mem_loc), &out_desc);
    return nri.fromC(api.MemoryDesc, out_desc);
}

// MapBuffer: ?*const fn (buffer: ?*Buffer, offset: u64, size: u64) callconv(.c) ?*anyopaque = null,
pub fn map(self: Buffer, device: *const nri.Device, offset: u64, size: u64) ![]u8 {
    const raw_ptr = device.interface.core.MapBuffer.?(self.buffer, offset, size) orelse return err.Result.OutOfMemory;
    return @as([*]u8, @ptrCast(raw_ptr))[0..size];
}

// UnmapBuffer: ?*const fn (buffer: ?*Buffer) callconv(.c) void = null,
pub fn unmap(self: Buffer, device: *const nri.Device) void {
    device.interface.core.UnmapBuffer.?(self.buffer);
}

// GetBufferDeviceAddress: ?*const fn (buffer: ?*const Buffer) callconv(.c) u64 = null,
pub fn getDeviceAddress(self: Buffer, device: *const nri.Device) usize {
    return device.interface.core.GetBufferDeviceAddress.?(self.buffer);
}

// GetBufferNativeObject: ?*const fn (buffer: ?*const Buffer) callconv(.c) u64 = null,
pub fn getNativeObject(self: Buffer, device: *const nri.Device) u64 {
    return device.interface.core.GetBufferNativeObject.?(self.buffer);
}

test {
    @import("std").testing.refAllDecls(@This());
}

//
