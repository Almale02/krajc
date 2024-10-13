const std = @import("std");
const ResourceState = @import("../../main.zig").ResourceState;
const uuid = @import("uuid");
const ecs = @import("../prelude.zig");

pub const SystemParam = struct {
    param_pos: usize,
    system_id: uuid.Uuid,
    resource_state: *ResourceState,
    last_run: ecs.Tick,
    alloc: std.mem.Allocator,
};
