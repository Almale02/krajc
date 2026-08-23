const std = @import("std");
const nri_c = @import("nri.zig");
const nri = @import("root.zig");
const Texture = @This();
const api = nri.api;
const err = @import("error.zig");

texture: *nri_c.Texture,

// GetTextureDesc: ?*const fn (texture: ?*const Texture) callconv(.c) [*c]const TextureDesc = null,
pub fn getDesc(self: Texture, device: *const nri.Device) SwapchainTextureDesc {
    const desc = device.interface.core.GetTextureDesc.?(self.texture);
    return nri.fromC(SwapchainTextureDesc, desc.*);
}

// DestroyTexture: ?*const fn (texture: ?*Texture) callconv(.c) void = null,
pub fn destroy(self: Texture, device: *const nri.Device) void {
    device.interface.core.DestroyTexture.?(self.texture);
}

// GetTextureMemoryDesc: ?*const fn (texture: ?*const Texture, memoryLocation: MemoryLocation, memoryDesc: [*c]MemoryDesc) callconv(.c) void = null,
pub fn getMemoryDesc(self: Texture, device: *const nri.Device, mem_loc: api.MemoryLocation) api.MemoryDesc {
    var out_desc: nri_c.MemoryDesc = undefined;
    device.interface.core.GetTextureMemoryDesc.?(self.texture, @intFromEnum(mem_loc), &out_desc);
    return nri.fromC(api.MemoryDesc, out_desc);
}

// GetTextureNativeObject: ?*const fn (texture: ?*const Texture) callconv(.c) u64 = null,
pub fn getNativeObject(self: Texture, device: *const nri.Device) u64 {
    return device.interface.core.GetTextureNativeObject.?(self.texture);
}

test {
    @import("std").testing.refAllDecls(@This());
    @import("std").testing.refAllDecls(SwapchainTextureDesc);
}

pub const SwapchainTextureDesc = struct {
    type: api.TextureType = api.TextureType.DEFAULT,
    usage: api.TextureUsageBits = .{},
    format: api.Format = api.Format.DEFAULT,
    width: u16 = 0,
    height: u16 = 0,
    depth: u16 = 0,
    mipNum: u16 = 0,
    layerNum: u16 = 0,
    sampleNum: u8 = 0,
    sharingMode: api.SharingMode = api.SharingMode.DEFAULT,
};
