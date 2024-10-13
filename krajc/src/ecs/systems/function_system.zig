const std = @import("std");
const uuid = @import("uuid");
const utils = @import("../../utils.zig");
const ecs = @import("../prelude.zig");
const ResourceState = @import("../../main.zig").ResourceState;
//const static_alloc = @import("../../main.zig").static_allocator;

pub fn FunctionSystem(comptime system_fn: anytype) type {
    const system_type = @TypeOf(system_fn);
    const system_info = @typeInfo(system_type);
    const Args = std.meta.ArgsTuple(system_type);
    //const system_id = uuid.v4.new();
    comptime {
        if (system_info != .Fn) @compileError("systems must be functions!");

        for (system_info.Fn.params) |param| {
            ensureSystemParam(param.type.?);
        }

        return struct {
            uuid: uuid.Uuid,

            const Self = @This();
            pub fn run(self: *const Self, resource: *ResourceState) void {
                const reg = resource.get(ecs.ArchetypeRegistry);

                const sys_infos = resource.get(SystemInfos);
                const info = sys_infos.systems.getOrPut(self.uuid) catch unreachable;

                if (!info.found_existing) {
                    info.value_ptr.* = SystemInfo{ .last_run = ecs.Tick.new(1, 0) };
                }

                var args: Args = undefined;
                inline for (std.meta.fields(Args), 0..) |field, i| {
                    const param_type = field.type;
                    //const param_info = @typeInfo(param_type);
                    const value = utils.accessMethod(param_type, "from_system_param", fn (ecs.SystemParam) param_type).?;
                    args[i] = value(.{ .param_pos = i, .system_id = self.uuid, .resource_state = resource, .alloc = @import("../../main.zig").static_allocator, .last_run = info.value_ptr.last_run });
                }
                @call(std.builtin.CallModifier.auto, system_fn, args);
                info.value_ptr.last_run = reg.curr_tick;
            }
            pub fn run_wrapper(ptr: *const anyopaque, resource: *ResourceState) void {
                const self: *const Self = @ptrCast(@alignCast(ptr));
                self.run(resource);
            }
            pub fn system(self: *const Self) ecs.System {
                return .{ .value = self, .run_ptr = Self.run_wrapper };
            }
        };
    }
}
fn ensureSystemParam(comptime T: type) void {
    comptime {
        if (!utils.ensureContainsMethod(
            T,
            "from_system_param",
            fn (ecs.SystemParam) T,
        )) {
            @compileError("all system params needs to implement from_system_param `fn(SystemParam) T`");
        }
    }
}

pub const SystemInfo = struct {
    last_run: ecs.Tick,
};
pub const SystemInfos = struct {
    alloc: std.mem.Allocator,
    systems: std.AutoHashMap(uuid.Uuid, SystemInfo) = undefined,

    pub fn default() SystemInfos {
        return SystemInfos{ .alloc = std.heap.page_allocator, .systems = std.AutoHashMap(uuid.Uuid, SystemInfo).init(std.heap.page_allocator) };
    }
};
