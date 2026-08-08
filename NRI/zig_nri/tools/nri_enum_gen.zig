const std = @import("std");

pub fn enum_gen(alloc: std.mem.Allocator, enums: []const []const u8, input_tree: *std.zig.Ast, output_writer: *std.Io.Writer) !void {
    for (enums) |enum_root| {
        const prefix = try std.fmt.allocPrint(alloc, "{s}_", .{enum_root});
        const root_decls = input_tree.rootDecls();
        var enum_variants: std.ArrayList(struct { name: []const u8, value: []const u8 }) = .empty;
        var enum_type_opt: ?[]const u8 = null;
        for (root_decls) |decl_idx| {
            const node = input_tree.nodes.get(@intFromEnum(decl_idx));
            if (node.tag != .simple_var_decl) continue;
            const decl_name = input_tree.tokenSlice(node.main_token + 1);
            if (std.mem.eql(u8, decl_name, enum_root)) {
                const node_opt = node.data.opt_node_and_opt_node;
                const decl_value = input_tree.getNodeSource(@enumFromInt(@intFromEnum(node_opt[1])));
                enum_type_opt = decl_value;
            }
            if (std.mem.startsWith(u8, decl_name, prefix) and decl_name.len > prefix.len) {
                const node_opt = node.data.opt_node_and_opt_node;
                const decl_type_name = input_tree.getNodeSource(@enumFromInt(@intFromEnum(node_opt[0])));
                _ = decl_type_name;
                const decl_value = input_tree.getNodeSource(@enumFromInt(@intFromEnum(node_opt[1])));
                try enum_variants.append(alloc, .{ .name = decl_name[prefix.len..], .value = decl_value });
            }
        }
        const enum_type = enum_type_opt orelse @panic("enum root wasnt found in the decls");
        try output_writer.print("pub const {[enum_name]s} = enum({[enum_type]s}) {{\n", .{ .enum_name = enum_root, .enum_type = enum_type });
        for (enum_variants.items) |variant| {
            try output_writer.print("\t{[name]s} = {[value]s},\n", .{ .name = variant.name, .value = variant.value });
        }
        try output_writer.writeAll("};\n");
    }
}
test {
    std.testing.refAllDecls(@This());
}
