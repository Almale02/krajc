const std = @import("std");
const krajc = @import("krajc");
const nri = @import("nri");
const sdl = @import("sdl");
pub fn main(init: std.process.Init) !void {
    _ = init;
    const a: f32 = 4.2;
    _ = a;
}

test {
    refAllDeclsRecursive(@This());
}

pub fn refAllDeclsRecursive(comptime T: type) void {
    if (!@import("builtin").is_test) return;

    const type_name = @typeName(T);
    if (!std.mem.startsWith(u8, type_name, "main")) {
        return;
    }
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
