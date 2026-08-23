const nri_c = @import("nri.zig");
const nri = @import("root.zig");
const Descriptor = @This();
const api = nri.api;
const err = @import("error.zig");

pub const DescriptorPool = struct {
    descriptor_pool: *nri_c.DescriptorPool,
    interface: *const nri.Interfaces,
    // DestroyDescriptorPool: ?*const fn (descriptorPool: ?*DescriptorPool) callconv(.c) void = null,
    pub fn destory(self: *DescriptorPool) void {
        self.interface.core.DestroyDescriptorPool.?(self.descriptor_pool);
    }

    // AllocateDescriptorSets: ?*const fn (descriptorPool: ?*DescriptorPool, pipelineLayout: ?*const PipelineLayout, setIndex: u32, descriptorSets: [*c]?*DescriptorSet, instanceNum: u32, variableDescriptorNum: u32) callconv(.c) Result = null,
    pub fn allocateSets(self: *DescriptorPool, pipeline_layout: *const api.PipelineLayout, set_idx: u32, descriptor_sets: []?DescriptorSet, variable_desc_num: u32) !void {
        try err.checkResultC(self.interface.core.AllocateDescriptorSets.?(
            self.descriptor_pool,
            pipeline_layout,
            set_idx,
            @ptrCast(descriptor_sets.ptr),
            @intCast(descriptor_sets.len),
            variable_desc_num,
        ));
    }

    // ResetDescriptorPool: ?*const fn (descriptorPool: ?*DescriptorPool) callconv(.c) void = null,
    pub fn reset(self: *DescriptorPool) void {
        self.interface.core.ResetDescriptorPool.?(self.descriptor_pool);
    }
};
pub const DescriptorSet = struct {
    descriptor_set: *nri_c.DescriptorSet,

    pub const DescriptorOffsets = struct { resource: u32 = 0, sampler: u32 = 0 };
    // GetDescriptorSetOffsets: ?*const fn (descriptorSet: ?*const DescriptorSet, resourceHeapOffset: [*c]u32, samplerHeapOffset: [*c]u32) callconv(.c) void = null,
    pub fn getOffsets(self: DescriptorSet, device: *const nri.Device) DescriptorOffsets {
        var offsets = DescriptorOffsets{};
        device.interface.core.GetDescriptorSetOffsets.?(self.descriptor_set, &offsets.resource, &offsets.sampler);
        return offsets;
    }
};

descriptor: *nri_c.Descriptor,

// DestroyDescriptor: ?*const fn (descriptor: ?*Descriptor) callconv(.c) void = null,
pub fn destroy(self: Descriptor, device: *const nri.Device) void {
    device.interface.core.DestroyDescriptor.?(self.descriptor);
}

// GetDescriptorNativeObject: ?*const fn (descriptor: ?*const Descriptor) callconv(.c) u64 = null,
pub fn getNativeObject(self: Descriptor, device: *const nri.Device) u64 {
    return device.interface.core.GetDescriptorNativeObject.?(self.descriptor);
}

test {
    @import("std").testing.refAllDecls(@This());
    @import("std").testing.refAllDecls(DescriptorPool);
    @import("std").testing.refAllDecls(DescriptorSet);
}
