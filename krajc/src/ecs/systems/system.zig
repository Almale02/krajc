const std = @import("std");
const uuid = @import("uuid");
const util = @import("../../utils.zig");
const resource_state = @import("../../main.zig").get_resource_state();
const ResourceState = @import("../../main.zig").ResourceState;
const Schedule = @import("schedule.zig").Schedule;
const FunctionSystem = @import("function_system.zig").FunctionSystem;

pub const System = struct {
    value: *const anyopaque,
    run_ptr: *const fn (*const anyopaque, *ResourceState) void,

    const Self = @This();

    pub fn run(self: *const Self, res: *ResourceState) void {
        self.run_ptr(self.value, res);
    }
};

pub fn register(Sched: type, system: anytype) void {
    //std.debug.print("ran register", .{});
    Schedule.ensureSchedule(Sched);
    const fn_sys = FunctionSystem(system){ .uuid = uuid.v4.new() };
    const sys: System = fn_sys.system();

    const res = resource_state.get(Sched);
    const registerer = util.accessMethod(Sched, "register", fn (*Sched, System) void) orelse unreachable;
    registerer(res, sys);
}
