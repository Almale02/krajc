const utils = @import("utils");
const wgpu = @import("wgpu");
const std = @import("std");
const krajc = @import("../../prelude.zig");
const uuid = @import("uuid");

pub const ByteSlice = struct {
    len: usize,
    ptr: [*]u8,
};
pub const FileLoader = struct {
    asset_manager: *krajc.AssetManager,
    path: []const u8,
    asset_id: uuid.Uuid,
    const Self = @This();
    const Result = struct {
        loader: *Self,
        alloc: std.mem.Allocator,
        handle: krajc.AssetHandle(ByteSlice),

        pub fn deinit(self: *Result) void {
            self.alloc.destroy(self.loader);
        }
    };

    pub fn start(asset_manager: *krajc.AssetManager, path: []const u8) Result {
        var mem = asset_manager.alloc.create(Self) catch unreachable;
        // using *[]u8 so that size of the slice is not lost
        const asset_handle = asset_manager.reserve_asset(ByteSlice);
        mem.* = Self{ .asset_manager = asset_manager, .path = path, .asset_id = asset_handle.id };
        asset_manager.chan.send(mem.generic()) catch unreachable;
        return Result{
            .loader = mem,
            .alloc = asset_manager.alloc,
            .handle = asset_handle,
        };
    }
    pub fn generic(self: *Self) krajc.AssetLoader {
        return krajc.AssetLoader{
            .ptr = self,
            .load_ptr = load_wrapper,
        };
    }

    pub fn load(self: *Self) void {
        const path = std.fs.cwd().realpathAlloc(self.asset_manager.alloc, self.path) catch |e| {
            std.debug.panic("failed to read file: {}, path was: {s}/{s}", .{ e, std.fs.cwd().realpathAlloc(self.asset_manager.alloc, ".") catch unreachable, self.path });
        };
        const file = std.fs.openFileAbsolute(path, .{ .mode = .read_only }) catch |e| {
            std.debug.panic("failed to read file: {}, path was: {s}", .{ e, path });
        };
        defer file.close();
        const file_len = file.getEndPos() catch unreachable;
        const bytes = self.asset_manager.alloc.alloc(u8, std.math.cast(usize, file_len).?) catch unreachable;
        _ = file.read(bytes) catch |e| {
            std.debug.panic("opened file, but failed to read content, err msg: {} ", .{e});
        };
        const slice = self.asset_manager.alloc.create(ByteSlice) catch unreachable;
        slice.* = ByteSlice{ .len = bytes.len, .ptr = bytes.ptr };

        const asset = self.asset_manager.entries.getPtr(self.asset_id) orelse std.debug.panic("failed to get asset based from asset id in load", .{});

        asset.ptr = @ptrCast(@alignCast(slice));
        asset.loaded = true;
    }
    fn load_wrapper(ptr: *anyopaque) void {
        const self: *Self = @ptrCast(@alignCast(ptr));

        self.load();
    }
};

pub const ShaderLoader = struct {
    asset_manager: *krajc.AssetManager,
    path: []const u8,
    device: *wgpu.Device,
    asset_id: uuid.Uuid,
    const Self = @This();
    const Result = struct {
        loader: *Self,
        alloc: std.mem.Allocator,
        handle: krajc.AssetHandle(wgpu.ShaderModule),

        pub fn deinit(self: *Result) void {
            self.alloc.destroy(self.loader);
        }
    };

    pub fn start(asset_manager: *krajc.AssetManager, device: *wgpu.Device, path: []const u8) Result {
        var mem = asset_manager.alloc.create(Self) catch unreachable;
        // using *[]u8 so that size of the slice is not lost
        const asset_handle = asset_manager.reserve_asset(wgpu.ShaderModule);
        mem.* = Self{ .asset_manager = asset_manager, .path = path, .asset_id = asset_handle.id, .device = device };
        asset_manager.chan.send(mem.generic()) catch unreachable;
        return Result{
            .loader = mem,
            .alloc = asset_manager.alloc,
            .handle = asset_handle,
        };
    }
    pub fn generic(self: *Self) krajc.AssetLoader {
        return krajc.AssetLoader{
            .ptr = self,
            .load_ptr = load_wrapper,
        };
    }

    pub fn load(self: *Self) void {
        const path = std.fs.cwd().realpathAlloc(self.asset_manager.alloc, self.path) catch |e| {
            std.debug.panic("failed to read file: {}, path was: {s}/{s}", .{ e, std.fs.cwd().realpathAlloc(self.asset_manager.alloc, ".") catch unreachable, self.path });
        };
        const file = std.fs.openFileAbsolute(path, .{ .mode = .read_only }) catch |e| {
            std.debug.panic("failed to read file: {}, path was: {s}", .{ e, path });
        };
        defer file.close();
        const file_len = file.getEndPos() catch unreachable;
        const bytes = self.asset_manager.alloc.alloc(u8, std.math.cast(usize, file_len).?) catch unreachable;
        _ = file.read(bytes) catch |e| {
            std.debug.panic("opened file, but failed to read content, err msg: {} ", .{e});
        };
        const module = self.device.createShaderModuleWGSL(
            "Shader module created by ShaderLoader",
            utils.makeNullTerminated(self.asset_manager.alloc, bytes),
        );

        const asset = self.asset_manager.entries.getPtr(self.asset_id) orelse std.debug.panic("failed to get asset based from asset id in load", .{});

        asset.ptr = @ptrCast(@alignCast(module));
        asset.loaded = true;
    }
    fn load_wrapper(ptr: *anyopaque) void {
        const self: *Self = @ptrCast(@alignCast(ptr));

        self.load();
    }
};
