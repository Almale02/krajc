const nri_c = @import("nri.zig");
const nri = @import("root.zig");
const Fence = @This();
const api = nri.api;
const err = @import("error.zig");

fence: *api.Fence,
interface: *const nri.Interfaces,

// DestroyFence: ?*const fn (fence: ?*Fence) callconv(.c) void = null,
pub fn destroy(self: *Fence) void {
    self.interface.core.DestroyFence.?(self.fence);
}

// Wait: ?*const fn (fence: ?*Fence, value: u64) callconv(.c) void = null,
pub fn wait(self: *Fence, value: u64) void {
    self.interface.core.Wait.?(self.fence, value);
}

// GetFenceValue: ?*const fn (fence: ?*Fence) callconv(.c) u64 = null,
pub fn getValue(self: *Fence) u64 {
    return self.interface.core.GetFenceValue.?(self.fence);
}

test {
    @import("std").testing.refAllDecls(@This());
}
