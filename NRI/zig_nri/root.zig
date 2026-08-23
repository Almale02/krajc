const std = @import("std");
pub const nri_c = @import("nri.zig");
pub const err = @import("error.zig");
pub const macro = @import("macro.zig");
pub const Device = @import("Device.zig");
pub const Fence = @import("Fence.zig");
pub const Descriptor = @import("Descriptor.zig");
pub const Pipeline = @import("Pipeline.zig");
pub const QueryPool = @import("QueryPool.zig");
pub const Buffer = @import("Buffer.zig");
pub const Texture = @import("Texture.zig");
pub const Queue = @import("Queue.zig");
pub const CommandBuffer = @import("CommandBuffer.zig");
pub const Swapchain = @import("Swapchain.zig");
pub const api = @import("codegen_out/codegen.zig");
pub const Memory = *api.Memory;

pub const Interfaces = struct {
    core: nri_c.CoreInterface,
    streamer: nri_c.StreamerInterface,
    swapchain: nri_c.SwapChainInterface,
    helper: nri_c.HelperInterface,
    imgui: nri_c.ImguiInterface,

    pub fn createFromDevice(device: *const nri_c.Device, interfaces: []const Interface) !Interfaces {
        return Interfaces{
            .core = Interface.getPtr(nri_c.CoreInterface, device) catch |e| b: {
                if (std.mem.find(Interface, interfaces, &[_]Interface{Interface.Core}) != null) {
                    std.debug.panic("failed to get core: {}", .{e});
                } else {
                    break :b std.mem.zeroes(nri_c.CoreInterface);
                }
            },
            .streamer = Interface.getPtr(nri_c.StreamerInterface, device) catch |e| b: {
                if (std.mem.find(Interface, interfaces, &[_]Interface{Interface.Streamer}) != null) {
                    std.debug.panic("failed to get streamer: {}", .{e});
                } else {
                    break :b std.mem.zeroes(nri_c.StreamerInterface);
                }
            },
            .swapchain = Interface.getPtr(nri_c.SwapChainInterface, device) catch |e| b: {
                if (std.mem.find(Interface, interfaces, &[_]Interface{Interface.Swapchain}) != null) {
                    std.debug.panic("failed to get swapchain: {}", .{e});
                } else {
                    break :b std.mem.zeroes(nri_c.SwapChainInterface);
                }
            },
            .helper = Interface.getPtr(nri_c.HelperInterface, device) catch |e| b: {
                if (std.mem.find(Interface, interfaces, &[_]Interface{Interface.Helper}) != null) {
                    std.debug.panic("failed to get helper: {}", .{e});
                } else {
                    break :b std.mem.zeroes(nri_c.HelperInterface);
                }
            },
            .imgui = Interface.getPtr(nri_c.ImguiInterface, device) catch |e| b: {
                if (std.mem.find(Interface, interfaces, &[_]Interface{Interface.Imgui}) != null) {
                    std.debug.panic("failed to get imgui: {}", .{e});
                } else {
                    break :b std.mem.zeroes(nri_c.ImguiInterface);
                }
            },
        };
    }
};
pub const Interface = enum {
    Core,
    Streamer,
    Swapchain,
    Helper,
    Imgui,

    // pub extern fn nriGetInterface(device: ?*const Device, interfaceName: [*c]const u8, interfaceSize: usize, interfacePtr: ?*anyopaque) Result;
    pub fn getPtr(comptime T: type, device: *const nri_c.Device) !T {
        var out_ptr: T = undefined;
        const interface = Interface.fromType(T);
        try err.checkResultC(nri_c.nriGetInterface(device, interface.getName().ptr, @sizeOf(T), &out_ptr));
        return out_ptr;
    }

    pub fn getName(self: Interface) [:0]const u8 {
        return switch (self) {
            .Core => "CoreInterface",
            .Streamer => "StreamerInterface",
            .Swapchain => "SwapChainInterface",
            .Helper => "HelperInterface",
            .Imgui => "ImguiInterface",
        };
    }
    pub fn fromType(comptime T: type) Interface {
        return switch (T) {
            nri_c.CoreInterface => Interface.Core,
            nri_c.StreamerInterface => Interface.Streamer,
            nri_c.SwapChainInterface => Interface.Swapchain,
            nri_c.HelperInterface => Interface.Helper,
            nri_c.ImguiInterface => Interface.Imgui,
            else => @compileError("unexpected interface"),
        };
    }
};
test {
    std.testing.refAllDecls(@This());
    std.testing.refAllDecls(Interface);
    _ = Interface.fromType(nri_c.SwapChainInterface);
    std.testing.refAllDecls(Interfaces);
    std.testing.refAllDecls(api);
}
pub fn convertVKFormatToNri(vkFormat: u32) api.Format {
    return @enumFromInt(nri_c.nriConvertVKFormatToNRI(vkFormat));
}

pub fn fromC(comptime T: type, c_val: anytype) T {
    const CType = @TypeOf(c_val);

    if (T == CType) return c_val;

    if (@typeInfo(T) == .@"enum") {
        return @enumFromInt(c_val);
    }
    if (@typeInfo(T) == .@"union") {
        @compileError("unions dont work");
    }

    if (@typeInfo(T) == .@"struct" and !@hasDecl(T, "translateC")) {
        if (@sizeOf(T) == @sizeOf(CType)) {
            return @bitCast(c_val);
        }
    }

    if (@typeInfo(T) == .@"struct") {
        var result: T = undefined;

        inline for (@typeInfo(T).@"struct".fields) |field| {
            if (@hasField(CType, field.name)) {
                @field(result, field.name) = fromC(field.type, @field(c_val, field.name));
            }
        }
        return result;
    }

    return @as(T, c_val);
}
