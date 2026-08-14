const std = @import("std");

pub fn bitfieldGen(arena: std.mem.Allocator, bitfields: []const []const u8, input_tree: *std.zig.Ast, output_writer: *std.Io.Writer) !void {
    for (bitfields) |bitfield_root| {
        const prefix = try std.fmt.allocPrint(arena, "{s}_", .{bitfield_root});
        const root_decls = input_tree.rootDecls();
        var bitfield_decls: std.ArrayList(struct { name: []const u8, value: []const u8 }) = .empty;
        var bitfield_type_opt: ?[]const u8 = null;
        for (root_decls) |decl_idx| {
            const node = input_tree.nodes.get(@intFromEnum(decl_idx));
            if (node.tag != .simple_var_decl) continue;
            const decl_name = input_tree.tokenSlice(node.main_token + 1);
            if (std.mem.eql(u8, decl_name, bitfield_root)) {
                const node_opt = node.data.opt_node_and_opt_node;
                const decl_value = input_tree.getNodeSource(@enumFromInt(@intFromEnum(node_opt[1])));
                bitfield_type_opt = decl_value;
            }
            if (std.mem.startsWith(u8, decl_name, prefix) and decl_name.len > prefix.len) {
                const node_opt = node.data.opt_node_and_opt_node;
                const decl_type_name = input_tree.getNodeSource(@enumFromInt(@intFromEnum(node_opt[0])));
                _ = decl_type_name;
                const decl_value = input_tree.getNodeSource(@enumFromInt(@intFromEnum(node_opt[1])));
                try bitfield_decls.append(arena, .{ .name = decl_name[prefix.len..], .value = decl_value });
            }
        }
        const bitfield_type = bitfield_type_opt orelse @panic("enum root wasnt found in the decls");
        const bitfield_size = try std.fmt.parseInt(u8, bitfield_type[1..], 10);
        try output_writer.print("pub const {[bitfield_name]s} = packed struct({[enum_type]s}) {{\n", .{ .bitfield_name = bitfield_root, .enum_type = bitfield_type });
        var bit_map: std.AutoHashMap(u32, []const u8) = .init(arena);
        var bit_decls: std.ArrayList(struct { name: []const u8 }) = .empty;
        var constant_decls: std.ArrayList(struct { name: []const u8, value: []const u8 }) = .empty;
        var biggest_bit_value: u32 = 0;
        for (bitfield_decls.items) |variant| {
            const value = try std.fmt.parseUnsigned(u32, variant.value, 10);
            if (value > 0 and std.math.isPowerOfTwo(value)) {
                if (value > biggest_bit_value) {
                    biggest_bit_value = value;
                }
                try bit_map.put(value, variant.name);
            } else {
                try constant_decls.append(arena, .{ .name = variant.name, .value = variant.value });
            }
        }
        for (0..std.math.log2(biggest_bit_value) + 1) |i| {
            if (bit_map.get(@intCast(std.math.pow(usize, 2, i)))) |decl_name| {
                try bit_decls.append(arena, .{ .name = decl_name });
            } else {
                try bit_decls.append(arena, .{ .name = try std.fmt.allocPrint(arena, "resvd_{}", .{i}) });
            }
        }
        for (constant_decls.items) |decl| {
            try output_writer.print("\tpub const {s}: {s} = {s};\n", .{ decl.name, bitfield_type, decl.value });
        }
        for (bit_decls.items) |decl| {
            try output_writer.print("\t{s}: bool = false,\n", .{decl.name});
        }
        const current_size = bit_decls.items.len;
        const pad_size = bitfield_size - current_size;
        if (pad_size != 0) {
            const pad_value = try std.fmt.allocPrint(arena, "u{}", .{pad_size});
            try output_writer.print("\t_: {s} = 0,\n", .{pad_value});
        }
        try output_writer.writeAll("};\n");
    }
}
test {
    std.testing.refAllDecls(@This());
}
