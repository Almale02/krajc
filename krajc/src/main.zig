const utils = @import("utils.zig");
const func_system = @import("ecs/systems/function_system.zig");

//const krajc_ecs = @import("krajc_ecs");

const zchan = @import("zchan");
const std = @import("std");
const zm = @import("math.zig");
const wgpu = @import("wgpu");
const glfw = @import("glfw");
const uuid = @import("uuid");
const event_callbacks = @import("glfw_event_callbacks.zig");
const ecs = @import("ecs/prelude.zig");
const register = ecs.register;
const schedule = ecs.schedule;
const Res = ecs.Res;
const Query = ecs.Query;
pub const TypeId = utils.TypeId;
pub const ResourceState = @import("engine_state/resource.zig").ResourceState;
pub const Event = @import("glfw_event_callbacks.zig").Event;
pub const EventResource = @import("glfw_event_callbacks.zig").EventResource;
pub const KeyInput = @import("engine_state/events/input.zig").KeyInput;
pub const MouseInput = @import("engine_state/events/input.zig").MouseInput;
pub const RenderingState = @import("prelude.zig").RenderingState;
pub const AssetManager = @import("engine_state/asset_manager.zig").AssetManager;
pub const IAssetLoader = @import("engine_state/asset_manager.zig").IAssetLoader;
pub const AssetHandle = @import("engine_state/asset_manager.zig").AssetHandle;
pub const FileLoader = @import("engine_state/asset_manager.zig").FileLoader;

const PlayerInfo = struct {
    health: u16,
    speed: f32,
    damage: f32,

    pub fn default() PlayerInfo {
        return PlayerInfo{ .health = 3, .speed = 1.5, .damage = 15.0 };
    }
};

var _arena_allocator = std.heap.ArenaAllocator.init(std.heap.page_allocator);
pub const static_allocator = _arena_allocator.allocator();

var resource_state = ResourceState{ .resources = std.AutoHashMap(TypeId, *anyopaque).init(static_allocator) };
pub fn get_resource_state() *ResourceState {
    return &resource_state;
}

fn error_callback(error_code: glfw.ErrorCode, description: [:0]const u8) void {
    std.debug.panic("glfw error: {}: {s}\n", .{ error_code, description });
}

pub fn main() !void {
    glfw.setErrorCallback(error_callback);
    try wgpu.Impl.init(static_allocator, .{});

    if (!glfw.init(.{})) {
        std.debug.panic("failed to init glfw", .{});
    }
    defer glfw.terminate();
    errdefer glfw.terminate();

    const asset_manager = resource_state.set(
        AssetManager,
        AssetManager.init(static_allocator),
    );
    var rendering_state = resource_state.set(RenderingState, .{});
    {
        rendering_state.window = glfw.Window.create(700, 500, "krajc window", null, null, .{ .client_api = .no_api }) orelse {
            std.debug.panic("failed to create window: {?s}", .{glfw.getErrorString()});
        };
    }
    var window = rendering_state.window;

    defer window.destroy();
    errdefer window.destroy();

    setup_event_callbacks(&window);

    const events_res = resource_state.get(EventResource);
    const key_input = resource_state.get(KeyInput);
    const mouse_input = resource_state.get(MouseInput);
    key_input.window = &window;
    mouse_input.window = &window;

    var reg = ecs.ArchetypeRegistry.init_ptr(static_allocator);
    reg = resource_state.set(ecs.ArchetypeRegistry, reg.*);
    defer reg.deinit();

    register(schedule.UpdateSchedule, system);

    reg.curr_tick = ecs.Tick.new(2, 0);

    try @import("rendering/data/rendering_state.zig").init_wgpu(window, rendering_state);

    const asset_loading_thread = std.Thread.spawn(.{}, AssetManager.run, .{asset_manager}) catch unreachable;
    _ = asset_loading_thread;
    //defer asset_loading_thread.join();

    var asset = FileLoader.start(asset_manager, "file.txt");
    defer asset.deinit();
    std.time.sleep(std.time.ns_per_ms * 2900);
    const loaded = asset.handle.is_loaded();
    const data = asset.handle.get().*.*;
    //_ = data;
    // not deadlock
    std.debug.print("ads", .{});
    // deadlock
    if (loaded) {
        //std.log.info("{}", .{data.ptr[0]});
        std.debug.print("{}", .{data.len});
        //std.debug.print("{any}", .{data});
    }

    //std.debug.print("data is: {s}", .{data});

    while (!window.shouldClose()) {
        //const loaded = asset.handle.is_loaded();
        //std.debug.print("loaded: {}\n", .{loaded});

        //std.debug.print("--------------\n", .{});
        key_input.step_frame();
        mouse_input.step_frame();

        glfw.pollEvents();
        const events = events_res.clear();

        for (events.items) |event| {
            switch (event) {
                .Key => |x| {
                    key_input.add_event(.{ .key = x.key, .action = x.action, .mods = x.mods });
                },
                .CursorMoved => |x| {
                    mouse_input.cursor_moved = .{ .Moved = zm.Vec2.new_f64(x.x_pos, x.y_pos) };
                },
                .MouseButton => |x| {
                    //std.debug.print("{}d mouse button {} with mods: {}\n", .{ x.action, x.button, x.mods });
                    mouse_input.add_event(.{ .button = x.button, .action = x.action, .mods = x.mods });
                },
                else => {},
            }
        }
        //std.debug.print("mouse state: {}d\n", .{window.getMouseButton(.left)});

        {
            const sched = resource_state.get(schedule.UpdateSchedule);
            sched.execute(&resource_state);
        }
        reg.inc_subtick();
        {
            const sched = resource_state.get(schedule.PostUpdateSchedule);
            sched.execute(&resource_state);
        }
        reg.inc_tick();

        try @import("rendering/systems/render.zig").render(&resource_state);
        //std.debug.print("--------------\n", .{});
    }
}
fn setup_event_callbacks(window: *glfw.Window) void {
    window.setCursorPosCallback(event_callbacks.cursor_moved_event_callback);
    window.setFramebufferSizeCallback(event_callbacks.resized_event_callback);
    window.setCloseCallback(event_callbacks.closed_event_callback);
    window.setFocusCallback(event_callbacks.focuse_event_callback);
    window.setKeyCallback(event_callbacks.key_event_callback);
    window.setMouseButtonCallback(event_callbacks.mouse_button_event_callback);
    window.setCursorEnterCallback(event_callbacks.cursor_enter_event_callback);
    window.setScrollCallback(event_callbacks.mouse_scroll_event_callback);
    window.setCharCallback(event_callbacks.char_event_callback);
}
const Transform = struct {
    x: f32,
    y: f32,
    z: f32,

    pub fn new(
        x: f32,
        y: f32,
        z: f32,
    ) Transform {
        return Transform{
            .x = x,
            .y = y,
            .z = z,
        };
    }
};

fn system(mouse: Res(MouseInput)) void {
    switch (mouse.ptr.cursor_moved) {
        .Moved => |x| {
            std.debug.print("cursor moved by: x: {d:.2}, y: {d:.2}\n", .{ x.x(), x.y() });
        },
        .None => {},
    }
}
