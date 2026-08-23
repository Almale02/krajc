const std = @import("std");
const nri_c = @import("nri.zig");
const nri = @import("root.zig");
const Device = @This();
const api = nri.api;
const err = @import("error.zig");

device: *nri_c.Device,
interface: *nri.Interfaces,

// pub extern fn nriCreateDevice(deviceCreationDesc: [*c]const DeviceCreationDesc, device: [*c]?*Device) Result;
pub fn createDevice(desc: api.DeviceCreationDesc, arena: std.mem.Allocator, requests: []const nri.Interface) !nri.Device {
    var out_device: ?*nri_c.Device = null;
    try err.checkResultC(nri_c.nriCreateDevice(&try desc.translateC(arena), &out_device));
    const interfaces = try nri.Interfaces.createFromDevice(out_device.?, requests);
    const interface_ptr = try arena.create(nri.Interfaces);
    interface_ptr.* = interfaces;
    return Device{ .device = out_device.?, .interface = interface_ptr };
}
pub fn createDeviceC(desc: *const nri_c.DeviceCreationDesc, arena: std.mem.Allocator) !*nri_c.Device {
    _ = arena;
    var out_device: ?*nri_c.Device = null;
    try err.checkResultC(nri_c.nriCreateDevice(desc, &out_device));
    return out_device.?;
}
// pub extern fn nriDestroyDevice(device: ?*Device) void;
pub fn destroy(self: *Device) void {
    nri_c.nriDestroyDevice(self.device);
}

// GetDeviceDesc: ?*const fn (device: ?*const NriDevice) callconv(.c) [*c]const NriDeviceDesc = null,
pub fn getDeviceDesc(self: *Device) ?api.DeviceDesc {
    if (self.interface.core.GetDeviceDesc.?(self.device)) |res| {
        return nri.fromC(api.DeviceDesc, res.?.*);
    }
    return null;
}

//GetFormatSupport: ?*const fn (device: ?*const NriDevice, format: NriFormat) callconv(.c) NriFormatSupportBits = null,
pub fn getFormatSupport(self: *Device, format: api.Format) api.FormatSupportBits {
    return @bitCast(self.interface.core.GetFormatSupport.?(self.device, @intFromEnum(format)));
}

//GetQueue: ?*const fn (device: ?*NriDevice, queueType: NriQueueType, queueIndex: u32, queue: [*c]?*NriQueue) callconv(.c) NriResult = null,
pub fn getQueue(self: *Device, queue_type: api.QueueType, idx: u32) !nri.Queue {
    var out_queue: ?*api.c.Queue = null;
    try err.checkResultC(self.interface.core.GetQueue.?(self.device, @intFromEnum(queue_type), idx, &out_queue));
    return nri.Queue{ .queue = out_queue.?, .interface = self.interface };
}

//CreateFence: ?*const fn (device: ?*NriDevice, initialValue: u64, fence: [*c]?*NriFence) callconv(.c) NriResult = null,
pub fn createSwapchainFence(self: *Device) !nri.Fence {
    var out_fence: ?*api.c.Fence = null;
    try err.checkResultC(self.interface.core.CreateFence.?(self.device, std.math.maxInt(u64), &out_fence));
    return nri.Fence{ .fence = out_fence.?, .interface = self.interface };
}
pub fn createTimelineFence(self: *Device, init_value: u64) !nri.Fence {
    var out_fence: ?*api.c.Fence = null;
    try err.checkResultC(self.interface.core.CreateFence.?(self.device, init_value, &out_fence));
    return nri.Fence{ .fence = out_fence.?, .interface = self.interface };
}

//CreateDescriptorPool: ?*const fn (device: ?*NriDevice, descriptorPoolDesc: [*c]const NriDescriptorPoolDesc, descriptorPool: [*c]?*NriDescriptorPool) callconv(.c) NriResult = null,
pub fn createDescriptorPool(self: *Device, descriptor_pool_desc: api.DescriptorPoolDesc) !nri.Descriptor.DescriptorPool {
    var out_pool: ?*api.c.DescriptorPool = null;
    try err.checkResultC(self.interface.core.CreateDescriptorPool.?(self.device, &descriptor_pool_desc.translateC(), &out_pool));
    return nri.Descriptor.DescriptorPool{ .descriptor_pool = out_pool.?, .interface = self.interface };
}

//CreatePipelineLayout: ?*const fn (device: ?*NriDevice, pipelineLayoutDesc: [*c]const NriPipelineLayoutDesc, pipelineLayout: [*c]?*NriPipelineLayout) callconv(.c) NriResult = null,
pub fn createPipelineLayout(self: *Device, arena: std.mem.Allocator, pipeline_layout_desc: api.PipelineLayoutDesc) !nri.Pipeline.PipelineLayout {
    var out_layout: ?*api.c.PipelineLayout = null;
    try err.checkResultC(self.interface.core.CreatePipelineLayout.?(self.device, &try pipeline_layout_desc.translateC(arena), &out_layout));
    return nri.Pipeline.PipelineLayout{ .pipeline_layout = out_layout.?, .interface = self.interface };
}

//CreateGraphicsPipeline: ?*const fn (device: ?*NriDevice, graphicsPipelineDesc: [*c]const NriGraphicsPipelineDesc, pipeline: [*c]?*NriPipeline) callconv(.c) NriResult = null,
pub fn createGraphicsPipeline(self: *Device, arena: std.mem.Allocator, graphics_pipeline_desc: api.GraphicsPipelineDesc) !nri.Pipeline {
    var out_pipeline: ?*api.c.Pipeline = null;
    try err.checkResultC(self.interface.core.CreateGraphicsPipeline.?(self.device, &try graphics_pipeline_desc.translateC(arena), &out_pipeline));
    return nri.Pipeline{ .pipeline = out_pipeline.?, .interface = self.interface };
}
//CreateComputePipeline: ?*const fn (device: ?*NriDevice, computePipelineDesc: [*c]const NriComputePipelineDesc, pipeline: [*c]?*NriPipeline) callconv(.c) NriResult = null,
pub fn createComputePipeline(self: *Device, graphics_pipeline_desc: api.ComputePipelineDesc) !nri.Pipeline {
    var out_pipeline: ?*api.c.Pipeline = null;
    try err.checkResultC(self.interface.core.CreateComputePipeline.?(self.device, &graphics_pipeline_desc.translateC(), &out_pipeline));
    return nri.Pipeline{ .pipeline = out_pipeline.?, .interface = self.interface };
}

//CreatePipelineCache: ?*const fn (device: ?*NriDevice, pipelineCacheDesc: [*c]const NriPipelineCacheDesc, pipelineCache: [*c]?*NriPipelineCache) callconv(.c) NriResult = null,
pub fn createPipelineCache(self: *Device, pipeline_cache_desc: api.PipelineCacheDesc) !nri.Pipeline.PipelineCache {
    var out_pipeline_cache: ?*api.c.PipelineCache = null;
    try err.checkResultC(self.interface.core.CreatePipelineCache.?(self.device, &pipeline_cache_desc.translateC(), &out_pipeline_cache));
    return nri.Pipeline.PipelineCache{ .pipeline_cache = out_pipeline_cache.?, .interface = self.interface };
}

//CreateQueryPool: ?*const fn (device: ?*NriDevice, queryPoolDesc: [*c]const NriQueryPoolDesc, queryPool: [*c]?*NriQueryPool) callconv(.c) NriResult = null,
pub fn createQueryPool(self: *Device, query_pool_desc: api.QueryPoolDesc) !nri.QueryPool {
    var out_query_pool: ?*api.c.QueryPool = null;
    try err.checkResultC(self.interface.core.CreateQueryPool.?(self.device, &query_pool_desc.translateC(), &out_query_pool));
    return nri.QueryPool{ .query_pool = out_query_pool.?, .interface = self.interface };
}

//CreateSampler: ?*const fn (device: ?*NriDevice, samplerDesc: [*c]const NriSamplerDesc, sampler: [*c]?*NriDescriptor) callconv(.c) NriResult = null,
pub fn createSampler(self: *Device, sampler_desc: api.SamplerDesc) !nri.Descriptor {
    var out_sampler: ?*api.c.Descriptor = null;
    try err.checkResultC(self.interface.core.CreateSampler.?(self.device, &sampler_desc.translateC(), &out_sampler));
    return nri.Descriptor{ .descriptor = out_sampler.? };
}

//AllocateMemory: ?*const fn (device: ?*NriDevice, allocateMemoryDesc: [*c]const NriAllocateMemoryDesc, memory: [*c]?*NriMemory) callconv(.c) NriResult = null,
pub fn allocateMemory(self: *Device, desc: api.AllocateMemoryDesc) !*api.Memory {
    var out_mem: ?*api.c.Memory = null;
    try err.checkResultC(self.interface.core.AllocateMemory.?(self.device, &desc.translateC(), &out_mem));
    return out_mem.?;
}

// FreeMemory: ?*const fn (memory: ?*Memory) callconv(.c) void = null,
pub fn freeMemory(self: *Device, mem: *api.Memory) void {
    self.interface.core.FreeMemory.?(mem);
}

//CreateBuffer: ?*const fn (device: ?*NriDevice, bufferDesc: [*c]const NriBufferDesc, buffer: [*c]?*NriBuffer) callconv(.c) NriResult = null,
pub fn createBuffer(self: *Device, buffer_desc: api.BufferDesc) !nri.Buffer {
    var out_buffer: ?*api.c.Buffer = null;
    try err.checkResultC(self.interface.core.CreateBuffer.?(self.device, &buffer_desc.translateC(), &out_buffer));
    return nri.Buffer{ .buffer = out_buffer.? };
}
//CreateTexture: ?*const fn (device: ?*NriDevice, textureDesc: [*c]const NriTextureDesc, texture: [*c]?*NriTexture) callconv(.c) NriResult = null,
pub fn createTexture(self: *Device, buffer_desc: api.TextureDesc) !nri.Texture {
    var out_texture: ?*api.c.Texture = null;
    try err.checkResultC(self.interface.core.CreateTexture.?(self.device, &buffer_desc.translateC(), &out_texture));
    return nri.Texture{ .texture = out_texture.? };
}

//GetBufferMemoryDesc2: ?*const fn (device: ?*const NriDevice, bufferDesc: [*c]const NriBufferDesc, memoryLocation: NriMemoryLocation, memoryDesc: [*c]NriMemoryDesc) callconv(.c) void = null,
pub fn getBufferMemoryDesc(self: *Device, buffer_desc: api.BufferDesc, mem_location: api.MemoryLocation) !api.MemoryDesc {
    var out_desc: api.c.MemoryDesc = undefined;
    self.interface.core.GetBufferMemoryDesc2.?(self.device, &buffer_desc.translateC(), @intFromEnum(mem_location), &out_desc);
    return nri.fromC(api.MemoryDesc, out_desc);
}

//GetTextureMemoryDesc2: ?*const fn (device: ?*const NriDevice, textureDesc: [*c]const NriTextureDesc, memoryLocation: NriMemoryLocation, memoryDesc: [*c]NriMemoryDesc) callconv(.c) void = null,
pub fn getTextureMemoryDesc(self: *Device, texture_desc: api.TextureDesc, mem_location: api.MemoryLocation) !api.MemoryDesc {
    var out_desc: api.c.MemoryDesc = undefined;
    self.interface.core.GetTextureMemoryDesc2.?(self.device, &texture_desc.translateC(), @intFromEnum(mem_location), &out_desc);
    return nri.fromC(api.MemoryDesc, out_desc);
}

//CreateCommittedBuffer: ?*const fn (device: ?*NriDevice, memoryLocation: NriMemoryLocation, priority: f32, bufferDesc: [*c]const NriBufferDesc, buffer: [*c]?*NriBuffer) callconv(.c) NriResult = null,
pub fn createCommitedBuffer(self: *Device, mem_loc: api.MemoryLocation, priority: f32, buffer_desc: api.BufferDesc) !nri.Buffer {
    var out_buff: ?*api.c.Buffer = null;
    try err.checkResultC(self.interface.core.CreateCommittedBuffer.?(self.device, @intFromEnum(mem_loc), priority, &buffer_desc.translateC(), &out_buff));
    return nri.Buffer{ .buffer = out_buff.? };
}

//CreateCommittedTexture: ?*const fn (device: ?*NriDevice, memoryLocation: NriMemoryLocation, priority: f32, textureDesc: [*c]const NriTextureDesc, texture: [*c]?*NriTexture) callconv(.c) NriResult = null,
pub fn createCommitedTexture(self: *Device, mem_loc: api.MemoryLocation, priority: f32, texture_desc: api.TextureDesc) !nri.Texture {
    var out_texture: ?*api.c.Texture = null;
    try err.checkResultC(self.interface.core.CreateCommittedTexture.?(self.device, @intFromEnum(mem_loc), priority, &texture_desc.translateC(), &out_texture));
    return nri.Texture{ .texture = out_texture.? };
}
//CreatePlacedBuffer: ?*const fn (device: ?*NriDevice, memory: ?*NriMemory, offset: u64, bufferDesc: [*c]const NriBufferDesc, buffer: [*c]?*NriBuffer) callconv(.c) NriResult = null,
pub fn createPlacedBuffer(self: *Device, mem: nri.Memory, offset: u64, buff_desc: api.BufferDesc) !nri.Buffer {
    var out_buffer: ?*api.c.Buffer = null;
    try err.checkResultC(self.interface.core.CreatePlacedBuffer.?(self.device, mem, offset, &buff_desc.translateC(), &out_buffer));
    return nri.Buffer{ .buffer = out_buffer.? };
}

//CreatePlacedTexture: ?*const fn (device: ?*NriDevice, memory: ?*NriMemory, offset: u64, textureDesc: [*c]const NriTextureDesc, texture: [*c]?*NriTexture) callconv(.c) NriResult = null,
pub fn createPlacedTexture(self: *Device, mem: nri.Memory, offset: u64, texture_desc: api.TextureDesc) !nri.Texture {
    var out_texture: ?*api.c.Texture = null;
    try err.checkResultC(self.interface.core.CreatePlacedTexture.?(self.device, mem, offset, &texture_desc.translateC(), &out_texture));
    return nri.Texture{ .texture = out_texture.? };
}

//DeviceWaitIdle: ?*const fn (device: ?*NriDevice) callconv(.c) NriResult = null,
pub fn waitIdle(self: *Device) !void {
    try err.checkResultC(self.interface.core.DeviceWaitIdle.?(self.device));
}
// CreateBufferView: ?*const fn (bufferViewDesc: [*c]const BufferViewDesc, bufferView: [*c]?*Descriptor) callconv(.c) Result = null, // here
pub fn createBufferView(self: *const Device, view_desc: api.BufferViewDesc) !nri.Descriptor {
    var out_view: ?*nri_c.Descriptor = null;
    try err.checkResultC(self.interface.core.CreateBufferView.?(&view_desc.translateC(), &out_view));
    return nri.Descriptor{ .descriptor = out_view.? };
}

// CreateTextureView: ?*const fn (textureViewDesc: [*c]const TextureViewDesc, textureView: [*c]?*Descriptor) callconv(.c) Result = null, // here
pub fn createTextureView(self: *const Device, view_desc: api.TextureViewDesc) !nri.Descriptor {
    var out_view: ?*nri_c.Descriptor = null;
    try err.checkResultC(self.interface.core.CreateTextureView.?(&view_desc.translateC(), &out_view));
    return nri.Descriptor{ .descriptor = out_view.? };
}

// BindBufferMemory: ?*const fn (bindBufferMemoryDescs: [*c]const BindBufferMemoryDesc, bindBufferMemoryDescNum: u32) callconv(.c) Result = null, // here
pub fn bindBufferMemory(self: *const Device, arena: std.mem.Allocator, bind_desc: []const api.BindBufferMemoryDesc) !void {
    const desc_buff = try arena.alloc(nri_c.BindBufferMemoryDesc, bind_desc.len);
    for (bind_desc, 0..) |desc, i| desc_buff[i] = desc.translateC();
    try err.checkResultC(self.interface.core.BindBufferMemory.?(desc_buff.ptr, @intCast(desc_buff.len)));
}

// BindTextureMemory: ?*const fn (bindTextureMemoryDescs: [*c]const BindTextureMemoryDesc, bindTextureMemoryDescNum: u32) callconv(.c) Result = null, // here
pub fn bindTextureMemory(self: *const Device, arena: std.mem.Allocator, bind_desc: []const api.BindTextureMemoryDesc) !void {
    const desc_buff = try arena.alloc(nri_c.BindTextureMemoryDesc, bind_desc.len);
    for (bind_desc, 0..) |desc, i| desc_buff[i] = desc.translateC();
    try err.checkResultC(self.interface.core.BindTextureMemory.?(desc_buff.ptr, @intCast(desc_buff.len)));
}

// UpdateDescriptorRanges: ?*const fn (updateDescriptorRangeDescs: [*c]const UpdateDescriptorRangeDesc, updateDescriptorRangeDescNum: u32) callconv(.c) void = null, // here
pub fn updateDescriptorRanger(self: *const Device, arena: std.mem.Allocator, range_desc: []const api.UpdateDescriptorRangeDesc) !void {
    const desc_buff = try arena.alloc(nri_c.UpdateDescriptorRangeDesc, range_desc.len);
    for (range_desc, 0..) |desc, i| desc_buff[i] = desc.translateC();
    self.interface.core.UpdateDescriptorRanges.?(desc_buff.ptr, @intCast(desc_buff.len));
}

// CopyDescriptorRanges: ?*const fn (copyDescriptorRangeDescs: [*c]const CopyDescriptorRangeDesc, copyDescriptorRangeDescNum: u32) callconv(.c) void = null,// here
pub fn copyDescriptorRanger(self: *const Device, arena: std.mem.Allocator, range_desc: []const api.CopyDescriptorRangeDesc) !void {
    const desc_buff = try arena.alloc(nri_c.CopyDescriptorRangeDesc, range_desc.len);
    for (range_desc, 0..) |desc, i| desc_buff[i] = desc.translateC();
    self.interface.core.CopyDescriptorRanges.?(desc_buff.ptr, @intCast(desc_buff.len));
}

//GetDeviceNativeObject: ?*const fn (device: ?*const NriDevice) callconv(.c) ?*anyopaque = null,
pub fn getNativeObjecet(self: *const Device) ?*anyopaque {
    return self.interface.core.GetDeviceNativeObject.?(self.device);
}
//// Swapchain
// CreateSwapChain: ?*const fn (device: ?*Device, swapChainDesc: [*c]const SwapChainDesc, swapChain: [*c]?*SwapChain) callconv(.c) Result = null,
pub fn createSwapchain(self: *Device, swapchian_desc: api.SwapChainDesc) !nri.Swapchain {
    var out_swapchain: ?*nri_c.SwapChain = null;
    try err.checkResultC(self.interface.swapchain.CreateSwapChain.?(self.device, &swapchian_desc.translateC(), &out_swapchain));
    return nri.Swapchain{ .swapchain = out_swapchain.?, .interface = self.interface };
}
//// Helper
// CalculateAllocationNumber: ?*const fn (device: ?*const Device, resourceGroupDesc: [*c]const ResourceGroupDesc) callconv(.c) u32 = null,
pub fn calculateAllocNumber(self: *const Device, resource_group_desc: api.ResourceGroupDesc) u32 {
    return self.interface.helper.CalculateAllocationNumber.?(self.device, &resource_group_desc.translateC());
}

// AllocateAndBindMemory: ?*const fn (device: ?*Device, resourceGroupDesc: [*c]const ResourceGroupDesc, allocations: [*c]?*Memory) callconv(.c) Result = null,
pub fn allocateAndBindMemory(self: *Device, resource_group_desc: api.ResourceGroupDesc, allocations: []?nri.Memory) !void {
    try err.checkResultC(self.interface.helper.AllocateAndBindMemory.?(self.device, &resource_group_desc.translateC(), @ptrCast(allocations.ptr)));
}

// QueryVideoMemoryInfo: ?*const fn (device: ?*const Device, memoryLocation: MemoryLocation, videoMemoryInfo: [*c]VideoMemoryInfo) callconv(.c) Result = null,
pub fn queryVideoMemInfo(self: *const Device, mem_loc: api.MemoryLocation) !api.VideoMemoryInfo {
    var out_info: nri_c.VideoMemoryInfo = undefined;
    try err.checkResultC(self.interface.helper.QueryVideoMemoryInfo.?(self.device, @intFromEnum(mem_loc), &out_info));
    return nri.fromC(api.VideoMemoryInfo, out_info);
}
pub fn getSupportedDepthFormat(self: *const Device, min_bits: u32, stencil: bool) api.Format {
    return nri.fromC(api.Format, nri_c.nriGetSupportedDepthFormat(&self.interface.core, self.device, min_bits, stencil));
}

test {
    @import("std").testing.refAllDecls(@This());
}
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
