const std = @import("std");
const glfw = @import("glfw");
const resource_state = @import("main.zig").get_resource_state();
const static_alloc = @import("main.zig").static_allocator();

pub fn cursor_moved_event_callback(window: glfw.Window, x_pos: f64, y_pos: f64) void {
    const event_resource = resource_state.get(EventResource);

    event_resource.add_event(.{ .CursorMoved = .{ .window = window, .x_pos = x_pos, .y_pos = y_pos } });
    //_ = window;
    //std.debug.print("cursor is moved to x: {d:.2}, y: {d:.2}\n", .{ x_pos, y_pos });

}
pub fn resized_event_callback(window: glfw.Window, width: u32, height: u32) void {
    const event_resource = resource_state.get(EventResource);

    event_resource.add_event(.{ .Resized = .{ .window = window, .width = width, .height = height } });
    //
}
pub fn closed_event_callback(window: glfw.Window) void {
    const event_resource = resource_state.get(EventResource);

    event_resource.add_event(.{ .Closed = .{ .window = window } });
    //
}
pub fn focuse_event_callback(window: glfw.Window, focused: bool) void {
    const event_resource = resource_state.get(EventResource);

    event_resource.add_event(.{ .Focuse = .{ .window = window, .focused = focused } });
    //
}
pub fn key_event_callback(window: glfw.Window, key: glfw.Key, scancode: i32, action: glfw.Action, mods: glfw.Mods) void {
    const event_resource = resource_state.get(EventResource);

    event_resource.add_event(.{ .Key = .{ .window = window, .key = key, .scancode = scancode, .action = action, .mods = mods } });
    //
}
pub fn mouse_button_event_callback(window: glfw.Window, button: glfw.MouseButton, action: glfw.Action, mods: glfw.Mods) void {
    const event_resource = resource_state.get(EventResource);

    event_resource.add_event(.{ .MouseButton = .{ .window = window, .button = button, .action = action, .mods = mods } });
    //
}
pub fn cursor_enter_event_callback(window: glfw.Window, entered: bool) void {
    const event_resource = resource_state.get(EventResource);

    event_resource.add_event(.{ .CursorEnter = .{ .window = window, .entered = entered } });
    //
}
pub fn mouse_scroll_event_callback(window: glfw.Window, x_offset: f64, y_offset: f64) void {
    const event_resource = resource_state.get(EventResource);

    event_resource.add_event(.{ .MouseScroll = .{ .window = window, .x_offset = x_offset, .y_offset = y_offset } });
    //
}
pub fn char_event_callback(window: glfw.Window, unicode_char: u21) void {
    const event_resource = resource_state.get(EventResource);

    event_resource.add_event(.{ .Char = .{ .window = window, .unicode_char = unicode_char } });
    //
}

pub const Event = union(enum) {
    CursorMoved: struct { window: glfw.Window, x_pos: f64, y_pos: f64 },
    Resized: struct { window: glfw.Window, width: u32, height: u32 },
    Closed: struct { window: glfw.Window },
    Focuse: struct { window: glfw.Window, focused: bool },
    Key: struct { window: glfw.Window, key: glfw.Key, scancode: i32, action: glfw.Action, mods: glfw.Mods },
    MouseButton: struct { window: glfw.Window, button: glfw.MouseButton, action: glfw.Action, mods: glfw.Mods },
    CursorEnter: struct { window: glfw.Window, entered: bool },
    MouseScroll: struct { window: glfw.Window, x_offset: f64, y_offset: f64 },
    Char: struct { window: glfw.Window, unicode_char: u21 },
};
pub const EventResource = struct {
    allocator: std.mem.Allocator = @import("main.zig").static_allocator,
    events: std.ArrayList(Event) = undefined,

    const Self = @This();

    pub fn add_event(self: *Self, event: Event) void {
        self.events.append(event) catch |e| {
            std.debug.panic("failed to add event to array list: {}", .{e});
        };
    }
    pub fn init(self: *Self) void {
        self.events = std.ArrayList(Event).init(self.allocator);
    }
    pub fn deinit(self: *Self) void {
        self.events.deinit();
    }
    pub fn clear(self: *Self) std.ArrayList(Event) {
        const new = self.events.clone() catch std.debug.panic("failed to clone events", .{});
        self.events.clearAndFree();
        return new;
    }
    pub fn default() Self {
        var self: Self = .{};

        self.init();

        return self;
    }
};
