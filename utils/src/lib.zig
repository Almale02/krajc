const std = @import("std");

pub const TypeId = u64;

pub fn clone_slice(T: type, slice: []T, alloc: std.mem.Allocator) ![]T {
    const new_slice = alloc.alloc(T, slice.len) catch |e| {
        return e;
    };

    for (slice, 0..) |val, i| {
        new_slice[i] = val;
    }
    return new_slice;
}
pub fn get_random() std.Random {
    var prng = std.rand.DefaultPrng.init(@intCast(std.time.milliTimestamp()));
    return prng.random();
}
pub fn get_random_seed(seed: comptime_int) std.Random {
    var prng = std.rand.DefaultPrng.init(@intCast(seed));
    return prng.random();
}

fn comptimeHash(str: []const u8) u64 {
    var hash: u64 = 5381;
    for (str) |c| {
        const casted_int: u64 = @intCast(c);
        const multiplied = @mulWithOverflow(hash, 33)[0];
        const added = @addWithOverflow(multiplied, casted_int)[0];
        hash = added;
    }
    return hash;
}

pub fn type_id(comptime T: type) TypeId {
    return comptimeHash(@typeName(T));
}
pub fn accessMethod(comptime T: type, name: []const u8, comptime Func: type) ?Func {
    comptime {
        if (ensureContainsMethod(T, name, Func)) {
            return @field(T, name);
        }
        return null;
    }
}

pub fn ensureContainsMethod(comptime on_type: type, comptime name: []const u8, comptime func: type) bool {
    comptime {
        const funcInfo = @typeInfo(func);
        if (funcInfo != .Fn) {
            @compileError("`func` must be a function declaration or function pointer");
        }

        const funcName = name;
        const funcParams = funcInfo.Fn.params;
        const funcReturnType = funcInfo.Fn.return_type;

        const typeInfo = @typeInfo(on_type);
        if (typeInfo != .Struct) {
            return false;
        }

        // Check if the struct has the method
        const hasMethod = @hasDecl(on_type, funcName);
        if (!hasMethod) {
            return false;
        }

        // Retrieve the method's type info
        const method = @field(on_type, funcName);
        const methodInfo = @typeInfo(@TypeOf(method));
        if (methodInfo != .Fn) {
            return false;
        }

        const methodParams = methodInfo.Fn.params;
        const methodReturnType = methodInfo.Fn.return_type;

        for (methodParams, funcParams) |m_param, f_param| {
            if (m_param.type != f_param.type) {
                return false;
            }
        }

        return methodReturnType == funcReturnType;
    }
}
