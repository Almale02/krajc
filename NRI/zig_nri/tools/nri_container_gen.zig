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
    var opaques: std.BufSet = .init(arena);
    var parse_type_data_map = std.StringHashMap(ParseFieldType).init(arena);
    var translate_needs_alloc = std.BufSet.init(arena);
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
                    .arena = arena_parse.allocator(),
                    .tree = input_tree,
                    .arenaFieldType = arena,
                    .numbers = numbers,
                    .enums = enums,
                    .bitfields = bitfields,
                    .primitive_map = primitive_map,
                    .parse_type_data_map = parse_type_data_map,
                };
                defer arena_parse.deinit();

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
                    else => unreachable,
                };
                try parse_type_data_map.put(type_decl_name, decl_type);
                if (std.mem.endsWith(u8, decl_name, "Interface") or std.mem.endsWith(u8, decl_name, "AllocationCallbacks")) {
                    continue;
                }
                var buff: [2]std.zig.Ast.Node.Index = undefined;
                const fields = input_tree.fullContainerDecl(&buff, @enumFromInt(value_node_idx)) orelse @panic("couldnt find container decl");
                var parsed_fields = std.ArrayList(struct { field_type: ParseFieldType, name: []const u8 }).empty;
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
                                .arena = arena_parse.allocator(),
                                .tree = input_tree,
                                .arenaFieldType = arena,
                                .numbers = numbers,
                                .enums = enums,
                                .bitfields = bitfields,
                                .primitive_map = primitive_map,
                                .parse_type_data_map = parse_type_data_map,
                            };
                            const field_type_data = try parse_field_type_ctx.parseFieldType(field_type_idx, false);
                            _ = arena_parse.reset(.retain_capacity);
                            try parsed_fields.append(arena, .{ .field_type = field_type_data, .name = field_name });
                            if (field.ast.value_expr.unwrap()) |field_init_idx| {
                                _ = field_init_idx;
                                // structs
                                // const field_init = input_tree.getNodeSource(field_init_idx);
                                // const type_buff = try alloc.alloc(u8, 1024 * 3);

                                // try output_writer.print("\t{s}: {s} = {s}\n", .{ field_name, field_type_data, field_init });
                                // try output_writer.print("\t{s}: ", .{field_name});
                                // try field_type_data.format(output_writer);
                                // try output_writer.print(" = {s}\n", .{field_init});
                            } else {
                                // unions
                                // try output_writer.print("union {s}\n", .{decl_name});
                                // try output_writer.print("\t{s}: {s}\n", .{ field_name, field_type_str });
                            }
                        },
                        else => {},
                    }
                }
                var parsed_field_map = std.StringHashMap(ParseFieldType).init(arena);
                for (parsed_fields.items) |field| {
                    try parsed_field_map.put(field.name, field.field_type);
                }
                var slice_len_map = std.StringHashMap([]const u8).init(arena);
                switch (decl_type) {
                    .Struct, .UnnamedStruct => |struct_name| {
                        var new_fields: std.ArrayList(struct { field_type: ParseFieldType, name: []const u8, value: []const u8 }) = .empty;
                        var i: usize = 0;
                        while (i < parsed_fields.items.len) {
                            const curr_type = parsed_fields.items[i].field_type;
                            const curr_name = parsed_fields.items[i].name;
                            switch (curr_type) {
                                .Number => {
                                    try new_fields.append(arena, .{ .name = curr_name, .field_type = curr_type, .value = "0" });
                                },
                                .Bool => {
                                    try new_fields.append(arena, .{ .name = curr_name, .field_type = curr_type, .value = "false" });
                                },
                                .Anyopaque, .SliceNull, .ManyItemNull => {
                                    unreachable;
                                },

                                .Struct, .Bitfield, .UnnamedStruct, .UnnamedUnion => {
                                    try new_fields.append(arena, .{ .name = curr_name, .field_type = curr_type, .value = ".{}" });
                                },
                                .Array => |arr| {
                                    try new_fields.append(arena, .{ .name = curr_name, .field_type = curr_type, .value = try std.fmt.allocPrint(arena, ".{{0}} ** {s}", .{arr.size}) });
                                },
                                .Union => {
                                    try new_fields.append(arena, .{ .name = curr_name, .field_type = curr_type, .value = try std.fmt.allocPrint(arena, "{f}.default()", .{curr_type}) });
                                },
                                .Enum => |x| {
                                    const init_value = try std.fmt.allocPrint(arena, "{s}.DEFAULT", .{x});
                                    try new_fields.append(arena, .{ .name = curr_name, .field_type = curr_type, .value = init_value });
                                },

                                .PointerC => |x| {
                                    if (if (i + 1 < parsed_fields.items.len) parsed_fields.items[i + 1] else null) |next_field| {
                                        const is_next_u32 = switch (next_field.field_type) {
                                            .Number => |num| std.mem.eql(u8, num, "u32"),
                                            else => false,
                                        };

                                        if (is_next_u32 and std.mem.endsWith(u8, next_field.name, "Num")) {
                                            i += 1;
                                            try new_fields.append(arena, .{ .name = curr_name, .field_type = ParseFieldType{ .SliceNull = .{ .ptr = x.ptr, .is_const = x.is_const } }, .value = "null" });
                                            try slice_len_map.put(curr_name, next_field.name);
                                        } else {
                                            try new_fields.append(arena, .{ .name = curr_name, .field_type = curr_type, .value = "null" });
                                        }
                                    } else {
                                        try new_fields.append(arena, .{ .name = curr_name, .field_type = curr_type, .value = "null" });
                                    }
                                },
                                .PointerNull => {
                                    try new_fields.append(arena, .{ .name = curr_name, .field_type = curr_type, .value = "null" });
                                },
                            }
                            i += 1;
                        }
                        try output_writer.print("pub const {s} = struct {{\n", .{struct_name});
                        for (new_fields.items) |new_field| {
                            const field_type = new_field.field_type;
                            const field_name = new_field.name;
                            const field_value = new_field.value;

                            try output_writer.print("\t{s}: {f} = {s},\n", .{ field_name, field_type, field_value });
                        }

                        var translate_fields: std.ArrayList(TranslateTypeContext.NewField) = .empty;
                        var translate_ctx = TranslateTypeContext{
                            .arena = arena,
                            .opaques = &opaques,
                            .translate_needs_alloc = translate_needs_alloc,
                            .slice_len_map = slice_len_map,
                            .parsed_field_map = parsed_field_map,
                            .new_fields = &translate_fields,
                            .needs_alloc = false,
                            .block_idx = 0,
                        };
                        for (new_fields.items) |field| {
                            const field_ref = try std.fmt.allocPrint(arena, "self.{s}", .{field.name});
                            _ = try translate_ctx.gen_translate(field.field_type, field.name, field_ref);
                        }

                        if (translate_ctx.needs_alloc) {
                            try output_writer.print("\tpub fn translateC(self: {[struct_name]s}, arena: std.mem.Allocator) !c.{[struct_name]s} {{\n", .{ .struct_name = struct_name });
                            try translate_needs_alloc.insert(struct_name);
                        } else {
                            try output_writer.print("\tpub fn translateC(self: {[struct_name]s}) c.{[struct_name]s} {{\n", .{ .struct_name = struct_name });
                        }
                        try output_writer.print("\t\treturn c.{s} {{\n", .{struct_name});

                        for (translate_fields.items) |field| {
                            try output_writer.print("\t\t\t.{s} = {s},\n", .{ field.field_name, field.translate });
                        }
                        try output_writer.writeAll("\t\t};\n");
                        try output_writer.writeAll("\t}\n");
                        try output_writer.writeAll("};\n");
                    },

                    // .Union, .UnnamedUnion => |union_name| {},
                    else => {},
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
                try output_writer.print("pub const {s} = c.{s};\n", .{ type_decl_name, type_decl_name });
                try opaques.insert(type_decl_name);
            },
            else => {
                //
            },
        }
    }
}
/// struct Test {
///     fn translateC(self: Test) c.Test {
///         return c.Test {num: self.num}
///     }
/// }
///
///
///
///
///
///
///
///
///
///
const TranslateTypeContext = struct {
    arena: std.mem.Allocator,
    new_fields: *std.ArrayList(NewField),
    parsed_field_map: std.StringHashMap(ParseFieldType),
    slice_len_map: std.StringHashMap([]const u8),
    translate_needs_alloc: std.BufSet,
    opaques: *std.BufSet,
    needs_alloc: bool = false,
    block_idx: u16,
    pub const NewField = struct { field_name: []const u8, translate: []const u8 };

    /// If top level, returns null, otherwise returns the translate string
    /// field_name: contains the field name if it is top level, otherwise null
    pub fn gen_translate(ctx: *TranslateTypeContext, field_type: ParseFieldType, field_name: ?[]const u8, value_ref: []const u8) !?[]const u8 {
        var allocating: std.Io.Writer.Allocating = .init(ctx.arena);
        defer allocating.deinit();
        var writer = &allocating.writer;
        switch (field_type) {
            .Number, .Bool, .Array => {
                try writer.writeAll(value_ref);
                if (field_name) |name| {
                    try ctx.new_fields.append(ctx.arena, NewField{ .field_name = name, .translate = try allocating.toOwnedSlice() });
                    return null;
                } else {
                    return try allocating.toOwnedSlice();
                }
            },
            .Struct, .UnnamedStruct, .UnnamedUnion, .Union => |decl_name| {
                if (field_name) |name| {
                    if (std.mem.eql(u8, name, "callbackInterface")) {
                        try writer.print("{s}", .{value_ref});
                    } else {
                        if (ctx.translate_needs_alloc.contains(decl_name)) {
                            try writer.print("try {s}.translateC(arena)", .{value_ref});
                            ctx.needs_alloc = true;
                        } else {
                            try writer.print("{s}.translateC()", .{value_ref});
                        }
                    }
                    try ctx.new_fields.append(ctx.arena, NewField{ .field_name = name, .translate = try allocating.toOwnedSlice() });
                    return null;
                } else {
                    if (ctx.translate_needs_alloc.contains(decl_name)) {
                        try writer.print("try {s}.translateC(arena)", .{value_ref});
                        ctx.needs_alloc = true;
                    } else {
                        try writer.print("{s}.translateC()", .{value_ref});
                    }
                    return try allocating.toOwnedSlice();
                }
            },
            .Enum => {
                try writer.print("@intFromEnum({s})", .{value_ref});

                if (field_name) |name| {
                    try ctx.new_fields.append(ctx.arena, NewField{ .field_name = name, .translate = try allocating.toOwnedSlice() });
                    return null;
                } else {
                    return try allocating.toOwnedSlice();
                }
            },
            .Bitfield => {
                try writer.print("@bitCast({s})", .{value_ref});

                if (field_name) |name| {
                    try ctx.new_fields.append(ctx.arena, NewField{ .field_name = name, .translate = try allocating.toOwnedSlice() });
                    return null;
                } else {
                    return try allocating.toOwnedSlice();
                }
            },
            .PointerC, .PointerNull => |x| {
                const is_opaque = switch (x.ptr.*) {
                    .Struct => |struct_name| ctx.opaques.contains(struct_name),
                    else => false,
                };
                if (x.ptr.is_primitive() or is_opaque) {
                    try writer.writeAll(value_ref);
                    if (field_name) |name| {
                        try ctx.new_fields.append(ctx.arena, NewField{ .field_name = name, .translate = try allocating.toOwnedSlice() });
                        return null;
                    } else {
                        return try allocating.toOwnedSlice();
                    }
                } else {
                    // const a: ?i32 = null;
                    // const res: ?i16 = if (a) |z| @intCast(z) else null;
                    const name = field_name orelse unreachable;
                    ctx.needs_alloc = true;
                    const value_derefed = try std.fmt.allocPrint(ctx.arena, "{s}.*", .{value_ref});
                    const inner = try ctx.gen_translate(x.ptr.*, null, value_derefed) orelse unreachable;
                    const parsed_inner_type = switch (ctx.parsed_field_map.get(name).?) {
                        .PointerC, .PointerNull => |inner_ptr| inner_ptr.ptr.*,
                        else => unreachable,
                    };
                    try writer.print(
                        \\b_{[block_idx]}: {{
                        \\if ({[value_ref]s} == null) {{break :b_{[block_idx]} null;}}
                        \\const value_{[block_idx]} = {[value]s};
                        \\const ptr_{[block_idx]} = try arena.create(c.{[c_type]f});
                        \\ptr_{[block_idx]}.* = value_{[block_idx]};
                        \\break :b_{[block_idx]} ptr_{[block_idx]};
                        \\}}
                    , .{ .block_idx = ctx.block_idx, .value = inner, .value_ref = value_ref, .c_type = parsed_inner_type });
                    try ctx.new_fields.append(ctx.arena, NewField{ .field_name = name, .translate = try allocating.toOwnedSlice() });
                    return null;
                }
            },
            .SliceNull => |x| {
                const name = field_name orelse unreachable;
                const is_opaque = switch (x.ptr.*) {
                    .Struct => |struct_name| ctx.opaques.contains(struct_name),
                    else => false,
                };
                if (x.ptr.is_primitive() or is_opaque) {
                    // try writer.print("{s}.ptr", .{value_ref});
                    try writer.print("if ({s}) |i| i.ptr else null", .{value_ref});
                    try ctx.new_fields.append(ctx.arena, NewField{ .field_name = name, .translate = try allocating.toOwnedSlice() });
                } else {
                    ctx.needs_alloc = true;
                    const value_access = try std.fmt.allocPrint(ctx.arena, "value_{}", .{ctx.block_idx});
                    const inner = try ctx.gen_translate(x.ptr.*, null, value_access) orelse unreachable;

                    const parsed_inner_type = switch (ctx.parsed_field_map.get(name).?) {
                        .PointerC => |inner_ptr| inner_ptr.ptr.*,
                        else => unreachable,
                    };
                    try writer.print(
                        \\b_{[block_idx]}: {{
                        \\if ({[value_ref]s} == null) {{break :b_{[block_idx]} null;}}
                        \\const ptr_{[block_idx]} = try arena.alloc(c.{[c_type]f}, {[value_ref]s}.?.len);
                        \\for ({[value_ref]s}.?, 0..) |value_{[block_idx]}, i_{[block_idx]}| {{
                        \\ptr_{[block_idx]}[i_{[block_idx]}] = {[value]s};
                        \\}}
                        \\break :b_{[block_idx]} ptr_{[block_idx]}.ptr;
                        \\}}
                    , .{ .block_idx = ctx.block_idx, .value = inner, .value_ref = value_ref, .c_type = parsed_inner_type });
                    try ctx.new_fields.append(ctx.arena, NewField{ .field_name = name, .translate = try allocating.toOwnedSlice() });
                }
                try writer.print("if ({[name]s} == null) 0 else @intCast({[name]s}.?.len)", .{ .name = value_ref });
                try ctx.new_fields.append(ctx.arena, NewField{ .field_name = ctx.slice_len_map.get(name).?, .translate = try allocating.toOwnedSlice() });
                return null;
            },
            .ManyItemNull => {
                unreachable;
            },
            .Anyopaque => {
                unreachable;
            },
        }
        unreachable;
    }
};

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
                        return .{ .PointerC = .{ .ptr = child, .is_const = ptr_type.const_token != null } };
                    },
                    .one => {
                        if (!nullable_curr) @panic("expected single item ptr to be nullable");
                        return .{ .PointerNull = .{ .ptr = child, .is_const = is_const } };
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
    PointerC: PointerData,
    PointerNull: PointerData,
    SliceNull: PointerData,
    ManyItemNull: PointerData,
    Array: struct { size: []const u8, item: *const ParseFieldType },
    UnnamedStruct: []const u8,
    UnnamedUnion: []const u8,

    pub const PointerData = struct { ptr: *const ParseFieldType, is_const: bool };
    pub fn is_primitive(self: ParseFieldType) bool {
        return switch (self) {
            .Number, .Bool, .Anyopaque, .Bitfield, .PointerNull, .PointerC => true,

            else => false,
        };
    }
    pub fn format(
        self: ParseFieldType,
        writer: *std.Io.Writer,
    ) std.Io.Writer.Error!void {
        switch (self) {
            .Number, .Struct, .Union, .Enum, .Bitfield, .UnnamedStruct, .UnnamedUnion => |x| {
                try writer.writeAll(x);
            },
            .Bool => {
                try writer.writeAll("bool");
            },
            .Anyopaque => {
                try writer.writeAll("anyopaque");
            },
            .PointerC => |x| {
                try writer.writeAll("[*c]");
                try x.ptr.format(writer);
            },
            .PointerNull => |x| {
                try writer.writeAll("?*");
                if (x.is_const) {
                    try writer.writeAll("const ");
                }
                try x.ptr.format(writer);
            },
            .SliceNull => |x| {
                try writer.writeAll("?[]");
                if (x.is_const) {
                    try writer.writeAll("const ");
                }
                try x.ptr.format(writer);
            },
            .ManyItemNull => |x| {
                try writer.writeAll("?[*]");
                if (x.is_const) {
                    try writer.writeAll("const ");
                }
                try x.ptr.format(writer);
            },

            .Array => |x| {
                try writer.writeAll("[");
                try writer.writeAll(x.size);
                try writer.writeAll("]");
                try x.item.format(writer);
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
