const std = @import("std");
const utils = @import("../utils.zig");
const static_allocator = @import("../main.zig").static_allocator;
const TypeId = utils.TypeId;

pub const ResourceState = struct {
    resources: std.AutoHashMap(TypeId, *anyopaque),

    const Self = @This();

    pub fn get_ref(self: *Self, comptime T: type) ?*T {
        const type_id = utils.type_id(T);

        // Safely unwrap the optional result from the HashMap
        const resource_ptr = self.resources.get(type_id) orelse return null;

        // Cast it to the correct type with alignment
        const value: *T = @ptrCast(@alignCast(resource_ptr));
        return value;
    }
    pub fn set(self: *Self, comptime T: type, value: T) *T {
        const type_id = utils.type_id(T);
        const resource = self.get_ref(T);
        if (resource == null) {
            const memory = static_allocator.create(T) catch std.debug.panic("failed to allocate memory", .{});

            memory.* = value;

            self.resources.put(type_id, memory) catch std.debug.panic("failed to allocate memory", .{});
        }
        if (resource) |res| {
            return res;
        }
        return self.get_ref(T).?;
    }
    pub fn get(self: *Self, comptime T: type) *T {
        if (self.get_ref(T)) |res| {
            return res;
        } else {
            const func = utils.accessMethod(T, "default", fn () T);
            if (func) |f| {
                return self.set(T, f());
            } else {
                return self.set(T, undefined);
            }
        }
    }
};
