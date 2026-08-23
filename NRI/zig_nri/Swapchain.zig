const std = @import("std");
const nri_c = @import("nri.zig");
const nri = @import("root.zig");
const Swapchain = @This();
const api = nri.api;
const err = @import("error.zig");

swapchain: *nri_c.SwapChain,
interface: *const nri.Interfaces,

// DestroySwapChain: ?*const fn (swapChain: ?*SwapChain) callconv(.c) void = null,
pub fn destroy(self: *Swapchain) void {
    self.interface.swapchain.DestroySwapChain.?(self.swapchain);
}

// GetSwapChainTextures: ?*const fn (swapChain: ?*const SwapChain, textureNum: [*c]u32) callconv(.c) [*c]const ?*Texture = null,
pub fn getTextures(self: *const Swapchain) []const nri.Texture {
    var out_num: u32 = undefined;
    const texture_ptr: ?[*]const nri.Texture = @ptrCast(self.interface.swapchain.GetSwapChainTextures.?(self.swapchain, &out_num));
    std.debug.assert(texture_ptr != null);
    return texture_ptr.?[0..out_num];
}

// GetDisplayDesc: ?*const fn (swapChain: ?*SwapChain, displayDesc: [*c]DisplayDesc) callconv(.c) Result = null,
pub fn getDisplayDesc(self: *Swapchain) !api.DisplayDesc {
    var out_desc: nri_c.DisplayDesc = undefined;
    try err.checkResultC(self.interface.swapchain.GetDisplayDesc.?(self.swapchain, &out_desc));
    return nri.fromC(api.DisplayDesc, out_desc);
}

// AcquireNextTexture: ?*const fn (swapChain: ?*SwapChain, acquireSemaphore: ?*Fence, textureIndex: [*c]u32) callconv(.c) Result = null,
pub fn aquireNextTexture(self: *Swapchain, semaphore: *nri.Fence) !u32 {
    var out_idx: u32 = undefined;
    try err.checkResultC(self.interface.swapchain.AcquireNextTexture.?(self.swapchain, semaphore.fence, &out_idx));
    return out_idx;
}

// WaitForPresent: ?*const fn (swapChain: ?*SwapChain) callconv(.c) Result = null,
pub fn waitForPresent(self: *Swapchain) !void {
    try err.checkResultC(self.interface.swapchain.WaitForPresent.?(self.swapchain));
}

// QueuePresent: ?*const fn (swapChain: ?*SwapChain, releaseSemaphore: ?*Fence) callconv(.c) Result = null,
pub fn queuePresent(self: *Swapchain, releaseSemaphore: *nri.Fence) !void {
    try err.checkResultC(self.interface.swapchain.QueuePresent.?(self.swapchain, releaseSemaphore.fence));
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
