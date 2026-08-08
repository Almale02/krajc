const std = @import("std");
pub const nri_c = @import("nri.zig");
pub const err = @import("error.zig");
pub const macro = @import("macro.zig");
pub const Device = @import("Device.zig");

pub fn fns() void {
    const a: f32 = 3.2;
    _ = a;
}
pub const Interfaces = struct {
    core: nri_c.CoreInterface,
    streamer: nri_c.StreamerInterface,
    swapchain: nri_c.SwapChainInterface,
    helper: nri_c.HelperInterface,
    imgui: nri_c.ImguiInterface,
};
test {
    std.testing.refAllDecls(@This());
}
