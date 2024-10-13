const std = @import("std");
const zchan = @import("zchan");
const utils = @import("../utils.zig");
const uuid = @import("uuid");
const krajc = @import("../prelude.zig");

pub const AssetManager = struct {
    alloc: std.mem.Allocator,
    entries: std.AutoHashMap(uuid.Uuid, AssetEntrie),
    //
    chan: zchan.Chan(AssetLoader),

    pub const AssetEntrie = struct {
        ptr: *anyopaque,
        loaded: bool,
    };
    const Self = @This();

    pub fn init(alloc: std.mem.Allocator) Self {
        return Self{ .alloc = alloc, .entries = std.AutoHashMap(uuid.Uuid, AssetEntrie).init(alloc), .chan = zchan.Chan(AssetLoader).init(alloc) };
    }
    pub fn reserve_asset(self: *Self, T: type) AssetHandle(T) {
        const id = uuid.v4.new();

        self.entries.put(id, AssetEntrie{ .ptr = undefined, .loaded = false }) catch unreachable;
        return AssetHandle(T){ .asset_manager = self, .id = id };
    }
    pub fn run(self: *Self) void {
        while (true) {
            const loader = self.chan.recv() catch |e| {
                switch (e) {
                    zchan.ChanError.Closed => {
                        break;
                    },
                    else => {
                        //std.debug.panic("channel reading failed with error: {}", .{e});
                        unreachable;
                    },
                }
            };
            loader.load();
        }
    }
};
pub const IAssetLoader = struct {
    pub fn ensureAssetLoader(T: type) void {
        if (utils.accessMethod(T, "load", fn (*T) void) == null) {
            @compileError(std.fmt.comptimePrint(
                \\ {} is not an AssetLoader, asset loaders needs to implement the following methods:
                \\ pub fn load(*Self) void,
            , .{}));
        }
    }
};
pub const AssetLoader = struct {
    ptr: *anyopaque,
    load_ptr: *const fn (*anyopaque) void,

    pub fn load(self: *const AssetLoader) void {
        self.load_ptr(self.ptr);
    }
};
pub fn AssetHandle(T: type) type {
    return struct {
        asset_manager: *AssetManager,
        id: uuid.Uuid,
        const Self = @This();
        pub fn get(self: *const Self) *T {
            const ptr: *T = @ptrCast(@alignCast(self.asset_manager.entries.get(self.id).?.ptr));
            return ptr;
        }
        pub fn get_copy(self: *const Self) T {
            self.asset_manager.entries.get(self.id).?.ptr.*;
        }
        pub fn is_loaded(self: *const Self) bool {
            return self.asset_manager.entries.getPtr(self.id).?.loaded;
        }
    };
}

pub const FileLoader = struct {
    asset_manager: *AssetManager,
    path: []const u8,
    asset_id: uuid.Uuid,
    const Self = @This();
    const Result = struct {
        loader: *Self,
        alloc: std.mem.Allocator,
        // using *[]u8 so that size of the slice is not lost
        handle: AssetHandle(*[]u8),

        pub fn deinit(self: *Result) void {
            self.alloc.destroy(self.loader);
        }
    };

    pub fn start(asset_manager: *AssetManager, path: []const u8) Result {
        var mem = asset_manager.alloc.create(Self) catch unreachable;
        // using *[]u8 so that size of the slice is not lost
        const asset_handle = asset_manager.reserve_asset(*[]u8);
        mem.* = Self{ .asset_manager = asset_manager, .path = path, .asset_id = asset_handle.id };
        asset_manager.chan.send(mem.generic()) catch unreachable;
        return Result{
            .loader = mem,
            .alloc = asset_manager.alloc,
            .handle = asset_handle,
        };
    }
    pub fn generic(self: *Self) AssetLoader {
        return AssetLoader{
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
        const bytes_ptr = self.asset_manager.alloc.create([]u8) catch unreachable;
        bytes_ptr.* = bytes;

        const asset = self.asset_manager.entries.getPtr(self.asset_id) orelse std.debug.panic("failed to get asset based from asset id in load", .{});

        asset.ptr = @ptrCast(bytes_ptr);
        asset.loaded = true;
    }
    fn load_wrapper(ptr: *anyopaque) void {
        const self: *Self = @ptrCast(@alignCast(ptr));

        self.load();
    }
};
