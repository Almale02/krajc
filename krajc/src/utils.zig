const std = @import("std");
const wgpu = @import("wgpu");
const glfw = @import("glfw");
const objc = @import("objc_message.zig");
const utils = @import("utils");
pub const TypeId = utils.TypeId;
pub const clone_slice = utils.clone_slice;
pub const get_random = utils.get_random;
pub const get_random_seed = utils.get_random_seed;
pub const type_id = utils.type_id;
pub const accessMethod = utils.accessMethod;
pub const ensureContainsMethod = utils.ensureContainsMethod;

pub fn createSurfaceForWindow(
    instance: *wgpu.Instance,
    window: glfw.Window,
    comptime glfw_options: glfw.BackendOptions,
) !*wgpu.Surface {
    const glfw_native = glfw.Native(glfw_options);
    if (glfw_options.win32) {
        return instance.createSurface(&wgpu.Surface.Descriptor{
            .next_in_chain = .{
                .from_windows_hwnd = &.{
                    .hinstance = std.os.windows.kernel32.GetModuleHandleW(null).?,
                    .hwnd = glfw_native.getWin32Window(window),
                },
            },
        });
    } else if (glfw_options.x11) {
        return instance.createSurface(&wgpu.Surface.Descriptor{
            .next_in_chain = .{
                .from_xlib_window = &.{
                    .display = glfw_native.getX11Display(),
                    .window = glfw_native.getX11Window(window),
                },
            },
        });
    } else if (glfw_options.wayland) {
        return instance.createSurface(&wgpu.Surface.Descriptor{
            .next_in_chain = .{
                .from_wayland_surface = &.{
                    .display = glfw_native.getWaylandDisplay(),
                    .surface = glfw_native.getWaylandWindow(window),
                },
            },
        });
    } else if (glfw_options.cocoa) {
        const pool = try AutoReleasePool.init();
        defer AutoReleasePool.release(pool);

        const ns_window = glfw_native.getCocoaWindow(window);
        const ns_view = msgSend(ns_window, "contentView", .{}, *anyopaque); // [nsWindow contentView]

        // Create a CAMetalLayer that covers the whole window that will be passed to CreateSurface.
        msgSend(ns_view, "setWantsLayer:", .{true}, void); // [view setWantsLayer:YES]
        const layer = msgSend(objc.objc_getClass("CAMetalLayer"), "layer", .{}, ?*anyopaque); // [CAMetalLayer layer]
        if (layer == null) @panic("failed to create Metal layer");
        msgSend(ns_view, "setLayer:", .{layer.?}, void); // [view setLayer:layer]

        // Use retina if the window was created with retina support.
        const scale_factor = msgSend(ns_window, "backingScaleFactor", .{}, f64); // [ns_window backingScaleFactor]
        msgSend(layer.?, "setContentsScale:", .{scale_factor}, void); // [layer setContentsScale:scale_factor]

        return instance.createSurface(&wgpu.Surface.Descriptor{
            .next_in_chain = .{
                .from_metal_layer = &.{ .layer = layer.? },
            },
        });
    } else unreachable;
}

pub fn detectGLFWOptions() glfw.BackendOptions {
    const target = @import("builtin").target;
    if (target.isDarwin()) return .{ .cocoa = true };
    return switch (target.os.tag) {
        .windows => .{ .win32 = true },
        .linux => .{ .x11 = true, .wayland = true },
        else => .{},
    };
}

pub const AutoReleasePool = if (!@import("builtin").target.isDarwin()) opaque {
    pub fn init() error{OutOfMemory}!?*AutoReleasePool {
        return null;
    }

    pub fn release(pool: ?*AutoReleasePool) void {
        _ = pool;
        return;
    }
} else opaque {
    pub fn init() error{OutOfMemory}!?*AutoReleasePool {
        // pool = [NSAutoreleasePool alloc];
        var pool = msgSend(objc.objc_getClass("NSAutoreleasePool"), "alloc", .{}, ?*AutoReleasePool);
        if (pool == null) return error.OutOfMemory;

        // pool = [pool init];
        pool = msgSend(pool, "init", .{}, ?*AutoReleasePool);
        if (pool == null) unreachable;

        return pool;
    }

    pub fn release(pool: ?*AutoReleasePool) void {
        // [pool release];
        msgSend(pool, "release", .{}, void);
    }
};

pub fn msgSend(obj: anytype, sel_name: [:0]const u8, args: anytype, comptime ReturnType: type) ReturnType {
    const args_meta = @typeInfo(@TypeOf(args)).Struct.fields;

    const FnType = switch (args_meta.len) {
        0 => *const fn (@TypeOf(obj), ?*objc.SEL) callconv(.C) ReturnType,
        1 => *const fn (@TypeOf(obj), ?*objc.SEL, args_meta[0].type) callconv(.C) ReturnType,
        2 => *const fn (@TypeOf(obj), ?*objc.SEL, args_meta[0].type, args_meta[1].type) callconv(.C) ReturnType,
        3 => *const fn (@TypeOf(obj), ?*objc.SEL, args_meta[0].type, args_meta[1].type, args_meta[2].type) callconv(.C) ReturnType,
        4 => *const fn (@TypeOf(obj), ?*objc.SEL, args_meta[0].type, args_meta[1].type, args_meta[2].type, args_meta[3].type) callconv(.C) ReturnType,
        else => @compileError("Unsupported number of args"),
    };

    const func = @as(FnType, @ptrCast(&objc.objc_msgSend));
    const sel = objc.sel_getUid(@as([*c]const u8, @ptrCast(sel_name)));

    return @call(.auto, func, .{ obj, sel } ++ args);
}
pub inline fn requestAdapterCallback(
    context: *RequestAdapterResponse,
    status: wgpu.RequestAdapterStatus,
    adapter: ?*wgpu.Adapter,
    message: ?[*:0]const u8,
) void {
    context.* = RequestAdapterResponse{
        .status = status,
        .adapter = adapter,
        .message = message,
    };
}
pub const RequestAdapterResponse = struct {
    status: wgpu.RequestAdapterStatus,
    adapter: ?*wgpu.Adapter,
    message: ?[*:0]const u8,
};

pub inline fn printUnhandledErrorCallback(typ: wgpu.ErrorType, message: [*:0]const u8) void {
    switch (typ) {
        .validation => std.log.err("gpu: validation error: {s}\n", .{message}),
        .out_of_memory => std.log.err("gpu: out of memory: {s}\n", .{message}),
        .device_lost => std.log.err("gpu: device lost: {s}\n", .{message}),
        .unknown => std.log.err("gpu: unknown error: {s}\n", .{message}),
        else => unreachable,
    }
    std.process.exit(1);
}
