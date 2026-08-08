const std = @import("std");
const Io = std.Io;
const sdl = @import("sdl");
const nri = @import("nri");
const builtin = @import("builtin");

pub fn engine() !void {
    try sdl.openURL("https://google.com");
}
pub fn other_one() !void {
    const value: f32 = 3.2;
    _ = value;
}
test {
    refAllDeclsRecursive(@This());
}

pub fn refAllDeclsRecursive(comptime T: type) void {
    if (!@import("builtin").is_test) return;
    inline for (comptime std.meta.declarations(T)) |decl| {
        if (@TypeOf(@field(T, decl.name)) == type) {
            switch (@typeInfo(@field(T, decl.name))) {
                .@"struct", .@"enum", .@"union", .@"opaque" => refAllDeclsRecursive(@field(T, decl.name)),
                else => {},
            }
        }
        _ = &@field(T, decl.name);
    }
}
