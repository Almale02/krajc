const std = @import("std");
const ecs = @import("ecs");

const Transform = struct {
    x: f32,
    y: f32,
    z: f32,

    pub fn new(x: f32, y: f32, z: f32) Transform {
        return .{ .x = x, .y = y, .z = z };
    }
};
const Tag = struct {};
pub fn main() void {
    var arena = std.heap.ArenaAllocator.init(std.heap.page_allocator);
    defer arena.deinit();
    const alloc = arena.allocator();

    var registry = ecs.ArchetypeRegistry.init_ptr(alloc);
    defer registry.deinit();

    const player1 = registry.new_entity();
    const player2 = registry.new_entity();
    const player3 = registry.new_entity();

    registry.add_component(player1, Transform.new(0.0, 0.5, 1.0)).?;
    registry.add_component(player2, Transform.new(1.0, 0.5, 1.0)).?;
    registry.add_component(player3, Transform.new(2.0, 0.5, 1.0)).?;
    registry.add_component(player1, @as(u32, 1)).?;
    registry.add_component(player2, @as(u32, 2)).?;
    registry.add_component(player3, @as(u32, 3)).?;

    // change detection doesnt work if started with 0 tick
    // the 1th tick is the tick before all systems have run
    // so the last run tick for systems which have not run yet, is the 1th tick
    registry.inc_tick();
    registry.inc_tick();

    registry.add_component(player1, @as(f32, 2.4)) orelse unreachable;
    registry.add_component(player2, @as(f32, 3.4)) orelse unreachable;
    // registry.add_component(player2, @as(f32, 3.4)) orelse unreachable;

    {
        var view = ecs.View(&.{ *Transform, *const u32 }, &.{ecs.Added(f32)}).init(registry, ecs.Tick.new(1, 0));
        //std.debug.print("{}", .{registry.curr_tick});

        while (view.next()) |access| {
            const num = access.get_const(u32).*;

            const trans = access.get(Transform);

            trans.y += @as(f32, @floatFromInt(num));
        }
    }
    registry.inc_subtick();
    {
        var view = ecs.View(&.{*const Transform}, &.{ecs.Changed(Transform)}).init(registry, ecs.Tick.new(1, 0));

        while (view.next()) |access| {
            const trans = access.get_const(Transform);
            std.debug.print("{}\n", .{trans.*});
            std.debug.print("-------\n", .{});
        }
    }
    registry.inc_tick();
    {
        var view = ecs.View(&.{*const Transform}, &.{ecs.Changed(Transform)}).init(registry, ecs.Tick.new(2, 1));

        while (view.next()) |access| {
            const trans = access.get_const(Transform);
            std.debug.print("{}\n", .{trans.*});
            std.debug.print("-------\n", .{});
        }
    }
    registry.inc_subtick();
    registry.remove_component(player2, Transform);
    {
        var view = ecs.View(&.{*const u32}, &.{ecs.Removed(Transform)}).init(registry, ecs.Tick.new(1, 0));

        while (view.next()) |access| {
            const num = access.get_const(u32);

            std.debug.print("removed has num: {}", .{num.*});
        }
    }
}
