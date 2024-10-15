const std = @import("std");
const util = @import("../../utils.zig");
const static_alloc = @import("../../main.zig").static_allocator;
const ResourceState = @import("../../main.zig").ResourceState;
const System = @import("system.zig").System;

pub const Schedule = struct {
    /// `name` is only used for differetiating generated schedules, because otherwise zig will cache the returned type and use it for all the schedules created,
    /// resulting all schedules having the same type id, and so being the same result
    pub fn Mained(name: []const u8) type {
        return struct {
            systems: std.ArrayList(System) = undefined,

            const Self = @This();

            pub fn init(self: *Self) void {
                self.systems = std.ArrayList(System).init(static_alloc);
                // you have to put it here, because if you put it outside the generated struct then it will also cache it
                _ = name;
            }
            pub fn register(self: *Self, system: System) void {
                self.systems.append(system) catch unreachable;
            }
            pub fn execute(self: *Self, res: *ResourceState) void {
                for (self.systems.items) |system| {
                    system.run(res);
                }
            }
            pub fn default() Self {
                var self: Self = .{};
                self.init();
                return self;
            }
        };
    }
    pub fn ensureSchedule(T: type) void {
        comptime {
            //@compileLog(@typeName(T));
            //if (!(util.ensureContainsMethod(T, "register", fn (*T, System) void) and util.ensureContainsMethod(T, "execute", fn (*T) void))) {
            if (!util.ensureContainsMethod(T, "register", fn (*T, System) void)) {
                const msg =
                    \\{s} is not a schedule, if you want to implement you own schedules, then you need to make a resource with these methods:
                    \\fn register(self: *Self, system: System) void,
                    \\fn execute(self: *Self) void,
                    \\if you have implemented them, make sure they are public!
                ;
                @compileError(std.fmt.comptimePrint(msg, .{@typeName(T)}));
            }
        }
    }

    pub fn access_register(T: type) ?(fn (*T, System) void) {
        return util.accessMethod(T, "register", fn (*T, System) void);
    }
    pub fn access_execute(T: type) ?(fn (*T, *ResourceState) void) {
        return util.accessMethod(T, "execute", fn (*T, *ResourceState) void);
    }
};

pub const UpdateSchedule = Schedule.Mained("update");
pub const PostUpdateSchedule = Schedule.Mained("post_update");
