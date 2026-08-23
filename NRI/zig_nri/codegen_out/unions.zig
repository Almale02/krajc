pub const Color = union(enum) {
    f: Color32f,
    ui: Color32ui,
    i: Color32i,
    pub fn translateC(self: Color) c.Color {
        return switch (self) {
            .f => |val| c.Color{ .f = val.translateC() },
            .ui => |val| c.Color{ .ui = val.translateC() },
            .i => |val| c.Color{ .i = val.translateC() },
        };
    }
    pub fn default() Color {
        return Color{ .f = std.mem.zeroes(Color32f) };
    }
};
pub const ClearValue = union(enum) {
    depthStencil: DepthStencil,
    color: Color,

    pub fn translateC(self: ClearValue) c.ClearValue {
        return switch (self) {
            .depthStencil => |val| c.ClearValue{ .depthStencil = val.translateC() },
            .color => |val| c.ClearValue{ .color = val.translateC() },
        };
    }
    pub fn default() ClearValue {
        return .{ .color = Color.default() };
    }
};
pub const union_unnamed_27 = union(enum) {
    triangles: BottomLevelTrianglesDesc,
    aabbs: BottomLevelAabbsDesc,

    pub fn translateC(self: union_unnamed_27) c.union_unnamed_27 {
        return switch (self) {
            .triangles => |val| c.union_unnamed_27{ .triangles = val.translateC() },
            .aabbs => |val| c.union_unnamed_27{ .aabbs = val.translateC() },
        };
    }
};
pub const union_unnamed_28 = union {
    upscaler: UpscalerGuides,
    denoiser: DenoiserGuides,

    pub fn translateC(self: union_unnamed_28) c.union_unnamed_28 {
        return switch (self) {
            .upscaler => |val| c.union_unnamed_28{ .upscaler = val.translateC() },
            .denoiser => |val| c.union_unnamed_28{ .denoiser = val.translateC() },
        };
    }
};
pub const union_unnamed_29 = union {
    nis: NISSettings,
    fsr: FSRSettings,
    dlrr: DLRRSettings,

    pub fn translateC(self: union_unnamed_29) c.union_unnamed_29 {
        return switch (self) {
            .nis => |val| c.union_unnamed_29{ .upscaler = val.translateC() },
            .fsr => |val| c.union_unnamed_29{ .fsr = val.translateC() },
            .dlrr => |val| c.union_unnamed_29{ .dlrr = val.translateC() },
        };
    }
};
pub const CallbackInterface = c.CallbackInterface;
pub const AllocationCallbacks = struct {
    Allocate: ?*const fn (userArg: ?*anyopaque, size: usize, alignment: usize) callconv(.c) ?*anyopaque = null,
    Reallocate: ?*const fn (userArg: ?*anyopaque, memory: ?*anyopaque, size: usize, alignment: usize) callconv(.c) ?*anyopaque = null,
    Free: ?*const fn (userArg: ?*anyopaque, memory: ?*anyopaque) callconv(.c) void = null,
    userArg: ?*anyopaque = null,
    disable3rdPartyAllocationCallbacks: bool = false,
    pub fn translateC(self: AllocationCallbacks) c.AllocationCallbacks {
        return .{
            .Allocate = self.Allocate,
            .Reallocate = self.Reallocate,
            .Free = self.Free,
            .userArg = self.userArg,
            .disable3rdPartyAllocationCallbacks = self.disable3rdPartyAllocationCallbacks,
        };
    }
};
