const std = @import("std");
const builtin = std.builtin;
const nri_c = @import("root.zig").nri_c;

pub fn NriEnumGen(namespace: anytype, comptime root_type_name: []const u8) type {
    @setEvalBranchQuota(80000);
    const namespace_decls = @typeInfo(namespace).@"struct".decls;
    const RootType = @field(namespace, root_type_name);
    const prefix: []const u8 = root_type_name ++ "_";
    var field_count: u16 = 0;
    inline for (namespace_decls) |decl| {
        comptime if (!std.mem.startsWith(u8, decl.name, prefix)) {
            continue;
        };
        comptime if (decl.name.len <= root_type_name.len + 1) {
            continue;
        };
        field_count += 1;
    }
    comptime var enum_field_names: [field_count][]const u8 = undefined;
    comptime var enum_field_values: [field_count]RootType = undefined;

    var i: u16 = 0;
    inline for (namespace_decls) |decl| {
        comptime if (!std.mem.startsWith(u8, decl.name, prefix)) {
            continue;
        };
        comptime if (decl.name.len <= root_type_name.len + 1) {
            continue;
        };
        const variant_value = @field(namespace, decl.name);
        enum_field_names[i] = decl.name[prefix.len..];
        enum_field_values[i] = variant_value;
        i += 1;
    }
    return @Enum(RootType, builtin.Type.Enum.Mode.exhaustive, &enum_field_names, &enum_field_values);
}
