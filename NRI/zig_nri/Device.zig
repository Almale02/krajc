const nri_c = @import("nri.zig");
const nri = @import("root.zig");
const Device = @This();
const Format = nri.macro.NriEnumGen(nri_c, "Format");

device: *const nri_c.Device,
interface: nri.Interfaces,

//GetDeviceDesc: ?*const fn (device: ?*const NriDevice) callconv(.c) [*c]const NriDeviceDesc = null,
pub fn getDeviceDesc(self: *Device) ?*const nri_c.DeviceDesc {
    const res = self.interface.core.GetDeviceDesc.?(self.device);
    const format: Format = Format.UNKNOWN;
    _ = format;
    return res;
}

//GetFormatSupport: ?*const fn (device: ?*const NriDevice, format: NriFormat) callconv(.c) NriFormatSupportBits = null,
const FormatSupportBits = enum(u16) {
    UNSUPPORTED = 0,
    TEXTURE = 1,
    STORAGE_TEXTURE = 2,
    STORAGE_TEXTURE_ATOMICS = 4,
    COLOR_ATTACHMENT = 8,
    DEPTH_STENCIL_ATTACHMENT = 16,
    BLEND = 32,
    MULTISAMPLE_2X = 64,
    MULTISAMPLE_4X = 128,
    MULTISAMPLE_8X = 256,
    MULTISAMPLE_RESOLVE = 512,
    BUFFER = 1024,
    STORAGE_BUFFER = 2048,
    STORAGE_BUFFER_ATOMICS = 4096,
    VERTEX_BUFFER = 8192,
    STORAGE_READ_WITHOUT_FORMAT = 16384,
    STORAGE_WRITE_WITHOUT_FORMAT = 32768,
};
// pub fn getFormatSupport(self: *Device, format: nri_c.Format) FormatSupportBits {}

//GetQueue: ?*const fn (device: ?*NriDevice, queueType: NriQueueType, queueIndex: u32, queue: [*c]?*NriQueue) callconv(.c) NriResult = null,

//CreateFence: ?*const fn (device: ?*NriDevice, initialValue: u64, fence: [*c]?*NriFence) callconv(.c) NriResult = null,

//CreateDescriptorPool: ?*const fn (device: ?*NriDevice, descriptorPoolDesc: [*c]const NriDescriptorPoolDesc, descriptorPool: [*c]?*NriDescriptorPool) callconv(.c) NriResult = null,

//CreatePipelineLayout: ?*const fn (device: ?*NriDevice, pipelineLayoutDesc: [*c]const NriPipelineLayoutDesc, pipelineLayout: [*c]?*NriPipelineLayout) callconv(.c) NriResult = null,

//CreateGraphicsPipeline: ?*const fn (device: ?*NriDevice, graphicsPipelineDesc: [*c]const NriGraphicsPipelineDesc, pipeline: [*c]?*NriPipeline) callconv(.c) NriResult = null,

//CreateComputePipeline: ?*const fn (device: ?*NriDevice, computePipelineDesc: [*c]const NriComputePipelineDesc, pipeline: [*c]?*NriPipeline) callconv(.c) NriResult = null,

//CreatePipelineCache: ?*const fn (device: ?*NriDevice, pipelineCacheDesc: [*c]const NriPipelineCacheDesc, pipelineCache: [*c]?*NriPipelineCache) callconv(.c) NriResult = null,

//CreateQueryPool: ?*const fn (device: ?*NriDevice, queryPoolDesc: [*c]const NriQueryPoolDesc, queryPool: [*c]?*NriQueryPool) callconv(.c) NriResult = null,

//CreateSampler: ?*const fn (device: ?*NriDevice, samplerDesc: [*c]const NriSamplerDesc, sampler: [*c]?*NriDescriptor) callconv(.c) NriResult = null,

//AllocateMemory: ?*const fn (device: ?*NriDevice, allocateMemoryDesc: [*c]const NriAllocateMemoryDesc, memory: [*c]?*NriMemory) callconv(.c) NriResult = null,

//CreateBuffer: ?*const fn (device: ?*NriDevice, bufferDesc: [*c]const NriBufferDesc, buffer: [*c]?*NriBuffer) callconv(.c) NriResult = null,

//CreateTexture: ?*const fn (device: ?*NriDevice, textureDesc: [*c]const NriTextureDesc, texture: [*c]?*NriTexture) callconv(.c) NriResult = null,

//GetBufferMemoryDesc2: ?*const fn (device: ?*const NriDevice, bufferDesc: [*c]const NriBufferDesc, memoryLocation: NriMemoryLocation, memoryDesc: [*c]NriMemoryDesc) callconv(.c) void = null,

//GetTextureMemoryDesc2: ?*const fn (device: ?*const NriDevice, textureDesc: [*c]const NriTextureDesc, memoryLocation: NriMemoryLocation, memoryDesc: [*c]NriMemoryDesc) callconv(.c) void = null,

//CreateCommittedBuffer: ?*const fn (device: ?*NriDevice, memoryLocation: NriMemoryLocation, priority: f32, bufferDesc: [*c]const NriBufferDesc, buffer: [*c]?*NriBuffer) callconv(.c) NriResult = null,

//CreateCommittedTexture: ?*const fn (device: ?*NriDevice, memoryLocation: NriMemoryLocation, priority: f32, textureDesc: [*c]const NriTextureDesc, texture: [*c]?*NriTexture) callconv(.c) NriResult = null,

//CreatePlacedBuffer: ?*const fn (device: ?*NriDevice, memory: ?*NriMemory, offset: u64, bufferDesc: [*c]const NriBufferDesc, buffer: [*c]?*NriBuffer) callconv(.c) NriResult = null,

//CreatePlacedTexture: ?*const fn (device: ?*NriDevice, memory: ?*NriMemory, offset: u64, textureDesc: [*c]const NriTextureDesc, texture: [*c]?*NriTexture) callconv(.c) NriResult = null,

//DeviceWaitIdle: ?*const fn (device: ?*NriDevice) callconv(.c) NriResult = null,

//GetDeviceNativeObject: ?*const fn (device: ?*const NriDevice) callconv(.c) ?*anyopaque = null,

test {
    @import("std").testing.refAllDecls(@This());
}
