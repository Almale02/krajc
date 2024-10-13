const ecs = @import("../../prelude.zig");
pub fn Query(access: []const type, filter: []const type) type {
    return struct {
        view_state: ecs.View(access, filter),

        const Self = @This();

        pub fn from_system_param(param: ecs.SystemParam) Self {
            const reg = param.resource_state.get(ecs.ArchetypeRegistry);
            return Self{ .view_state = ecs.View(access, filter).init(reg, param.last_run) };
        }
        pub fn reset(self: *Self) void {
            self.view_state.reset();
        }
        pub fn next(self: *Self) ?ecs.View(access, filter).TypedAccess {
            return self.view_state.next();
        }
    };
}
