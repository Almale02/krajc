const std = @import("std");
const nri_c = @import("nri.zig");
const nri = @import("root.zig");
const QueryPool = @This();
const api = nri.api;
const err = @import("error.zig");

query_pool: *nri_c.QueryPool,
interface: *const nri.Interfaces,

// DestroyQueryPool: ?*const fn (queryPool: ?*QueryPool) callconv(.c) void = null,
pub fn destory(self: *QueryPool) void {
    self.interface.core.DestroyQueryPool.?(self.query_pool);
}

// ResetQueries: ?*const fn (queryPool: ?*QueryPool, offset: u32, num: u32) callconv(.c) void = null,
pub fn resetQueries(self: *QueryPool, offset: u32, num: u32) void {
    self.interface.core.ResetQueries.?(self.query_pool, offset, num);
}

// GetQuerySize: ?*const fn (queryPool: ?*const QueryPool) callconv(.c) u32 = null,
pub fn getSize(self: *QueryPool) u32 {
    return self.interface.core.GetQuerySize.?(self.query_pool);
}

test {
    @import("std").testing.refAllDecls(@This());
}
