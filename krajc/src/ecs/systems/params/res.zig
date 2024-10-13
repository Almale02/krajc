const utils = @import("../../../utils.zig");
const ecs = @import("../../prelude.zig");
const resource_state = @import("../../../main.zig").get_resource_state();
const static_allocator = @import("../../../main.zig").static_allocator;
const TypeId = utils.TypeId;

pub fn Res(comptime T: type) type {
    return struct {
        ptr: *T,
        const Self = @This();
        pub fn from_system_param(param: ecs.SystemParam) Self {
            _ = param;

            return Self{ .ptr = resource_state.get(T) };
        }
    };
}
