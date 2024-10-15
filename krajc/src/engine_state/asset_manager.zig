const std = @import("std");
const zchan = @import("zchan");
const utils = @import("../utils.zig");
const uuid = @import("uuid");
const krajc = @import("../prelude.zig");

pub const AssetManager = struct {
    alloc: std.mem.Allocator,
    entries: std.AutoHashMap(uuid.Uuid, AssetEntrie),
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
        pub fn is_loaded(self: *const Self) bool {
            return self.asset_manager.entries.getPtr(self.id).?.loaded;
        }
    };
}
