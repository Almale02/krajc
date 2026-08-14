const std = @import("std");
const ConfigType = @import("main.zig").ConfigType;

pub fn containerGen(arena: std.mem.Allocator, alloc: std.mem.Allocator, config: *const ConfigType, input_tree: *std.zig.Ast, output_writer: *std.Io.Writer) !void {
    var numbers: std.BufSet = .init(arena);
    const number_types = .{ i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize, f16, f32, f64, f128 };
    var primitive_map: std.StringHashMap([]const u8) = .init(arena);

    inline for (number_types) |t| {
        try numbers.insert(@typeName(t));
    }
    var enums: std.BufSet = .init(arena);
    var bitfields: std.BufSet = .init(arena);
    var parse_type_data_map = std.StringHashMap(ParseFieldType).init(arena);
    for (config.enums) |enum_name| {
        try enums.insert(enum_name);
    }
    for (config.bitfields) |bitfield_name| {
        try bitfields.insert(bitfield_name);
    }
    const root_decls = input_tree.rootDecls();

    for (root_decls) |decl_idx| {
        const node = input_tree.nodes.get(@intFromEnum(decl_idx));
        if (node.tag != .simple_var_decl) continue;
        const decl_name = input_tree.tokenSlice(node.main_token + 1);
        const node_opt = node.data.opt_node_and_opt_node;
        const value_node_idx: u32 = @intFromEnum(node_opt[1]);
        const value_node = input_tree.nodes.get(@intCast(value_node_idx));
        switch (value_node.tag) {
            .identifier => {
                const ident_name = input_tree.tokenSlice(value_node.main_token);
                if (std.mem.eql(u8, ident_name, "bool")) {
                    try parse_type_data_map.put(decl_name, .Bool);
                    try primitive_map.put(decl_name, ident_name);
                } else if (std.mem.eql(u8, ident_name, "anyopaque")) {
                    try parse_type_data_map.put(decl_name, .Anyopaque);
                    try primitive_map.put(decl_name, ident_name);
                }
                if (numbers.contains(ident_name) and !enums.contains(decl_name) and !bitfields.contains(decl_name)) {
                    try primitive_map.put(decl_name, ident_name);
                    try parse_type_data_map.put(decl_name, .{ .Number = ident_name });
                }
                if (enums.contains(decl_name)) {
                    try parse_type_data_map.put(decl_name, .{ .Enum = decl_name });
                }
                if (bitfields.contains(decl_name)) {
                    try parse_type_data_map.put(decl_name, .{ .Bitfield = decl_name });
                }
            },
            .optional_type => {
                var arena_parse: std.heap.ArenaAllocator = .init(alloc);
                const parse_field_type_ctx = ParseFieldTypeContext{
                    .arena = arena,
                    .tree = input_tree,
                    .arenaFieldType = arena_parse.allocator(),
                    .numbers = numbers,
                    .enums = enums,
                    .bitfields = bitfields,
                    .primitive_map = primitive_map,
                    .parse_type_data_map = parse_type_data_map,
                };
                arena_parse.deinit();

                try parse_type_data_map.put(decl_name, try parse_field_type_ctx.parseFieldType(@enumFromInt(value_node_idx), false));
            },
            .container_decl_trailing, .container_decl_two_trailing => {
                var decl_type: ParseFieldType = undefined;
                if (std.mem.startsWith(u8, decl_name, "struct_unnamed_")) {
                    decl_type = ParseFieldType{ .UnnamedStruct = decl_name };
                } else if (std.mem.startsWith(u8, decl_name, "union_unnamed_")) {
                    decl_type = ParseFieldType{ .UnnamedUnion = decl_name };
                } else if (std.mem.startsWith(u8, decl_name, "struct_Nri")) {
                    const x = "struct_Nri";
                    decl_type = ParseFieldType{ .Struct = decl_name[x.len..] };
                } else if (std.mem.startsWith(u8, decl_name, "union_Nri")) {
                    const x = "union_Nri";
                    decl_type = ParseFieldType{ .Union = decl_name[x.len..] };
                } else {
                    std.debug.panic("got a container decl with unexpected type: {s}", .{decl_name});
                }
                const type_decl_name = switch (decl_type) {
                    .Struct, .UnnamedStruct, .UnnamedUnion, .Union => |x| x,
                    else => @panic("unreachable"),
                };
                try parse_type_data_map.put(type_decl_name, decl_type);
                if (std.mem.endsWith(u8, decl_name, "Interface")) {
                    continue;
                }
                var buff: [2]std.zig.Ast.Node.Index = undefined;
                const fields = input_tree.fullContainerDecl(&buff, @enumFromInt(value_node_idx)) orelse @panic("couldnt find container decl");
                var field_types = std.ArrayList(ParseFieldType).empty;
                for (fields.ast.members) |field_idx| {
                    const field_node = input_tree.nodes.get(@intFromEnum(field_idx));
                    switch (field_node.tag) {
                        .container_field_init, .container_field => {
                            const field = input_tree.fullContainerField(field_idx) orelse continue;
                            const field_name = input_tree.tokenSlice(field_node.main_token);
                            const field_type_idx = field.ast.type_expr.unwrap().?;
                            const field_type_str = input_tree.getNodeSource(field_type_idx);
                            _ = field_type_str;

                            var arena_parse: std.heap.ArenaAllocator = .init(alloc);
                            const parse_field_type_ctx = ParseFieldTypeContext{
                                .arena = arena,
                                .tree = input_tree,
                                .arenaFieldType = arena_parse.allocator(),
                                .numbers = numbers,
                                .enums = enums,
                                .bitfields = bitfields,
                                .primitive_map = primitive_map,
                                .parse_type_data_map = parse_type_data_map,
                            };
                            const field_type_data = try parse_field_type_ctx.parseFieldType(field_type_idx, false);
                            _ = arena_parse.reset(.retain_capacity);
                            if (field.ast.value_expr.unwrap()) |field_init_idx| {
                                // structs
                                const field_init = input_tree.getNodeSource(field_init_idx);
                                // const type_buff = try alloc.alloc(u8, 1024 * 3);

                                // try output_writer.print("\t{s}: {s} = {s}\n", .{ field_name, field_type_data, field_init });
                                try output_writer.print("\t{s}: ", .{field_name});
                                try field_type_data.format(output_writer);
                                try output_writer.print(" = {s}\n", .{field_init});
                            } else {
                                // unions
                                // try output_writer.print("union {s}\n", .{decl_name});
                                // try output_writer.print("\t{s}: {s}\n", .{ field_name, field_type_str });
                            }
                        },
                        else => {},
                    }
                }
            },
            // opaques
            .container_decl_two => {
                var decl_type: ParseFieldType = undefined;
                if (std.mem.startsWith(u8, decl_name, "struct_unnamed_")) {
                    decl_type = ParseFieldType{ .UnnamedStruct = decl_name };
                } else if (std.mem.startsWith(u8, decl_name, "union_unnamed_")) {
                    decl_type = ParseFieldType{ .UnnamedUnion = decl_name };
                } else if (std.mem.startsWith(u8, decl_name, "struct_Nri")) {
                    const x = "struct_Nri";
                    decl_type = ParseFieldType{ .Struct = decl_name[x.len..] };
                } else if (std.mem.startsWith(u8, decl_name, "union_Nri")) {
                    const x = "union_Nri";
                    decl_type = ParseFieldType{ .Union = decl_name[x.len..] };
                } else if (std.mem.eql(u8, decl_name, "struct_ImDrawList")) {
                    decl_type = ParseFieldType{ .Struct = "ImDrawList" };
                } else if (std.mem.eql(u8, decl_name, "struct_ImTextureData")) {
                    decl_type = ParseFieldType{ .Struct = "ImTextureData" };
                } else {
                    std.debug.panic("got a container decl with unexpected type here: {s}", .{decl_name});
                }
                const type_decl_name = switch (decl_type) {
                    .Struct => |x| x,
                    else => @panic("unreachable"),
                };
                try parse_type_data_map.put(type_decl_name, decl_type);
            },
            else => {
                //
            },
        }
    }
}

const ParseFieldTypeContext = struct {
    arena: std.mem.Allocator,
    arenaFieldType: std.mem.Allocator,
    tree: *const std.zig.Ast,
    numbers: std.BufSet,
    enums: std.BufSet,
    bitfields: std.BufSet,
    primitive_map: std.StringHashMap([]const u8),
    parse_type_data_map: std.StringHashMap(ParseFieldType),

    pub fn parseFieldType(
        ctx: ParseFieldTypeContext,
        type_idx: std.zig.Ast.Node.Index,
        nullable_curr: bool,
    ) !ParseFieldType {
        switch (ctx.tree.nodeTag(type_idx)) {
            .ptr_type, .ptr_type_aligned => {
                const ptr_type = ctx.tree.fullPtrType(type_idx).?;
                const is_const = ptr_type.const_token != null;
                const child = try ctx.arenaFieldType.create(ParseFieldType);
                child.* = try ctx.parseFieldType(ptr_type.ast.child_type, false);

                switch (ptr_type.size) {
                    .c => {
                        if (nullable_curr) @panic("didnt expect c pointer to be nullable");
                        return .{ .PointerC = child };
                    },
                    .one => {
                        if (!nullable_curr) @panic("expected single item ptr to be nullable");
                        if (is_const) {
                            return .{ .PointerConstNull = child };
                        } else {
                            return .{ .PointerNull = child };
                        }
                    },
                    else => @panic("unreachable"),
                }
            },
            .optional_type => {
                const child_idx = ctx.tree.nodeData(type_idx).node;
                return try ctx.parseFieldType(child_idx, true);
            },
            .array_type => {
                if (nullable_curr) @panic("didnt expect array type to be nullable");
                const array = ctx.tree.arrayType(type_idx);
                const size = ctx.tree.tokenSlice(ctx.tree.nodeMainToken(array.ast.elem_count));
                const child = try ctx.arenaFieldType.create(ParseFieldType);

                child.* = try ctx.parseFieldType(array.ast.elem_type, false);

                return .{ .Array = .{ .size = size, .item = child } };
            },
            .identifier => {
                if (nullable_curr) @panic("didnt expect ident type to be nullable");
                const ident = ctx.tree.tokenSlice(ctx.tree.nodeMainToken(type_idx));
                if (std.mem.eql(u8, ident, "anyopaque")) {
                    return .{ .Anyopaque = {} };
                }
                if (std.mem.eql(u8, ident, "bool")) {
                    return .{ .Bool = {} };
                }
                if (ctx.numbers.contains(ident)) {
                    return .{ .Number = ident };
                }
                if (ctx.enums.contains(ident)) {
                    return .{ .Enum = ident };
                }
                if (ctx.bitfields.contains(ident)) {
                    return .{ .Bitfield = ident };
                }
                if (ctx.parse_type_data_map.get(ident)) |primitive| {
                    return primitive;
                }
                std.debug.panic("unreachable ident: {s}", .{ident});
            },
            .fn_proto => {
                return .{ .Fn = {} };
            },
            else => {
                std.debug.panic("unreachable else: {s}", .{@tagName(ctx.tree.nodeTag(type_idx))});
            },
        }

        @panic("unreachable");
    }
};

const ParseFieldType = union(enum) {
    Number: []const u8,
    Bool: void,
    Anyopaque: void,
    Struct: []const u8,
    Union: []const u8,
    Enum: []const u8,
    Bitfield: []const u8,
    PointerC: *const ParseFieldType,
    PointerNull: *const ParseFieldType,
    PointerConstNull: *const ParseFieldType,
    SliceNull: *const ParseFieldType,
    SliceConstNull: *const ParseFieldType,
    ManyItemNull: *const ParseFieldType,
    ManyItemConstNull: *const ParseFieldType,
    Array: struct { size: []const u8, item: *const ParseFieldType },
    UnnamedStruct: []const u8,
    UnnamedUnion: []const u8,
    Fn: void,

    pub fn format(
        self: ParseFieldType,
        writer: *std.Io.Writer,
    ) std.Io.Writer.Error!void {
        switch (self) {
            .Number => |x| {
                try writer.writeAll(x);
            },
            .Bool => {
                try writer.writeAll("bool");
            },
            .Anyopaque => {
                try writer.writeAll("anyopaque");
            },

            .Struct => |x| {
                try writer.writeAll("Struct(");
                try writer.writeAll(x);
                try writer.writeAll(")");
            },
            .Union => |x| {
                try writer.writeAll("Union(");
                try writer.writeAll(x);
                try writer.writeAll(")");
            },
            .Enum => |x| {
                try writer.writeAll("Enum(");
                try writer.writeAll(x);
                try writer.writeAll(")");
            },
            .Bitfield => |x| {
                try writer.writeAll("Bitfield(");
                try writer.writeAll(x);
                try writer.writeAll(")");
            },

            .PointerC => |x| {
                try writer.writeAll("[*c]");
                try x.format(writer);
            },
            .PointerNull => |x| {
                try writer.writeAll("?*");
                try x.format(writer);
            },
            .PointerConstNull => |x| {
                try writer.writeAll("?*const ");
                try x.format(writer);
            },
            .SliceNull => |x| {
                try writer.writeAll("?[]");
                try x.format(writer);
            },
            .SliceConstNull => |x| {
                try writer.writeAll("?[]const ");
                try x.format(writer);
            },
            .ManyItemNull => |x| {
                try writer.writeAll("?[*]");
                try x.format(writer);
            },
            .ManyItemConstNull => |x| {
                try writer.writeAll("?[*]const ");
                try x.format(writer);
            },

            .Array => |x| {
                try writer.writeAll("[");
                try writer.writeAll(x.size);
                try writer.writeAll("]");
                try x.item.format(writer);
            },

            .UnnamedStruct => |x| {
                try writer.writeAll("UnnamedStruct(");
                try writer.writeAll(x);
                try writer.writeAll(")");
            },
            .UnnamedUnion => |x| {
                try writer.writeAll("UnnamedUnion(");
                try writer.writeAll(x);
                try writer.writeAll(")");
            },
            .Fn => {
                try writer.writeAll("fn");
            },
        }
    }
};
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
