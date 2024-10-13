const std = @import("std");
const zm = @import("../../math.zig");
const glfw = @import("glfw");
const hashset = @import("hashset");
const static_alloc = @import("../../main.zig").static_allocator;
const Event = @import("../../main.zig").Event;

const CursorMoved = union(enum) { Moved: zm.Vec2, None };

pub const MouseInput = struct {
    pub const MouseEvent = struct {
        button: glfw.MouseButton,
        action: glfw.Action,
        mods: glfw.Mods,
    };
    curr_events: hashset.Set(MouseEvent) = undefined,
    window: *glfw.Window = undefined,
    cursor_moved: CursorMoved = CursorMoved.None,

    const Self = @This();

    pub fn init(self: *Self) void {
        self.curr_events = hashset.Set(MouseEvent).init(static_alloc);
    }
    pub fn deinit(self: *Self) void {
        self.curr_events.deinit();
    }

    pub fn default() Self {
        var self: Self = .{};
        self.init();
        return self;
    }

    pub fn add_event(self: *Self, event: MouseEvent) void {
        if (event.action == .repeat) {
            return;
        }
        _ = self.curr_events.add(event) catch std.debug.panic("failed to add key event", .{});
    }
    pub fn step_frame(self: *Self) void {
        self.cursor_moved = .None;
        self.curr_events.clearAndFree();
    }
    pub fn is_pressed_mod(self: *const Self, key: glfw.MouseButton, mods: glfw.Mods) bool {
        const event = MouseEvent{ .button = key, .action = glfw.Action.press, .mods = mods };
        if (self.curr_events.contains(event)) {
            return true;
        } else {
            return false;
        }
    }
    pub fn is_pressed(self: *const Self, key: glfw.MouseButton) bool {
        const event = MouseEvent{ .button = key, .action = glfw.Action.press, .mods = .{} };
        if (self.curr_events.contains(event)) {
            return true;
        } else {
            return false;
        }
    }
    pub fn is_released_mod(self: *const Self, key: glfw.MouseButton, mods: glfw.Mods) bool {
        const event = MouseEvent{ .button = key, .action = glfw.Action.release, .mods = mods };
        if (self.curr_events.contains(event)) {
            return true;
        } else {
            return false;
        }
    }
    pub fn is_released(self: *const Self, key: glfw.MouseButton) bool {
        const event = MouseEvent{ .button = key, .action = glfw.Action.release, .mods = .{} };
        if (self.curr_events.contains(event)) {
            return true;
        } else {
            return false;
        }
    }
    pub fn is_held_down(self: *const Self, button: glfw.MouseButton) bool {
        return self.window.getMouseButton(button) == .press;
    }
    pub fn get_mouse_pos(self: *const Self) glfw.Window.CursorPos {
        return self.window.getCursorPos();
    }
    pub fn get_mouse_moved(self: *const Self) CursorMoved {
        return self.cursor_moved;
    }
};

pub const KeyInput = struct {
    pub const KeyEvent = struct {
        key: glfw.Key,
        action: glfw.Action,
        mods: glfw.Mods,
    };
    curr_events: hashset.Set(KeyEvent) = undefined,
    window: *glfw.Window = undefined,

    const Self = @This();

    pub fn init(self: *Self) void {
        self.curr_events = hashset.Set(KeyEvent).init(static_alloc);
    }
    pub fn deinit(self: *Self) void {
        self.curr_events.deinit();
    }

    pub fn default() Self {
        var self: Self = .{};
        self.init();
        return self;
    }

    pub fn add_event(self: *Self, event: KeyEvent) void {
        if (event.action == .repeat) {
            return;
        }
        _ = self.curr_events.add(event) catch std.debug.panic("failed to add key event", .{});
    }
    pub fn step_frame(self: *Self) void {
        self.curr_events.clearAndFree();
    }
    pub fn is_pressed_mod(self: *const Self, key: glfw.Key, mods: glfw.Mods) bool {
        const event = KeyEvent{ .key = key, .action = glfw.Action.press, .mods = mods };
        if (self.curr_events.contains(event)) {
            return true;
        } else {
            return false;
        }
    }
    pub fn is_pressed(self: *const Self, key: glfw.Key) bool {
        const event = KeyEvent{ .key = key, .action = glfw.Action.press, .mods = .{} };
        if (self.curr_events.contains(event)) {
            return true;
        } else {
            return false;
        }
    }
    pub fn is_released_mod(self: *const Self, key: glfw.Key, mods: glfw.Mods) bool {
        const event = KeyEvent{ .key = key, .action = glfw.Action.release, .mods = mods };
        if (self.curr_events.contains(event)) {
            return true;
        } else {
            return false;
        }
    }
    pub fn is_released(self: *const Self, key: glfw.Key) bool {
        const event = KeyEvent{ .key = key, .action = glfw.Action.release, .mods = .{} };
        if (self.curr_events.contains(event)) {
            return true;
        } else {
            return false;
        }
    }
    pub fn is_held_down(self: *const Self, key: glfw.Key) bool {
        return self.window.getKey(key) == .press;
    }
};
