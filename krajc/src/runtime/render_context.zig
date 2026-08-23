const nri = @import("nri");
const std = @import("std");
const sdl = @import("sdl");
pub const RenderContext = struct {
    device: nri.Device,
    render_queue: nri.Queue,
    swapchain: nri.Swapchain,
    swapchain_textures: []const nri.Texture,
    swapchain_texture_views: []const nri.Descriptor,
    arena: std.mem.Allocator,
    alloc: std.mem.Allocator,

    pub fn fromSdl3(window: sdl.video.Window, arena: std.mem.Allocator, alloc: std.mem.Allocator) !RenderContext {
        const props = try window.getProperties();
        const wayland_surface = props.wayland_surface.?.value;
        const wayland_display = props.wayland_display.?.value;
        const size = try window.getSizeInPixels();
        const size_x = size.@"0";
        const size_y = size.@"1";

        const extensions_null = try sdl.vulkan.getInstanceExtensions();
        var extensions = try arena.alloc([*]u8, extensions_null.len);
        for (0..extensions_null.len) |i| {
            extensions[i] = @ptrCast(@constCast(extensions_null[i]));
        }

        var device = try RenderContext.initDevice(arena, extensions);
        const queue = try device.getQueue(.GRAPHICS, 0);

        const swapchain_desc = nri.api.SwapChainDesc{
            .window = .{ .wayland = .{ .display = wayland_display, .surface = wayland_surface } },
            .queue = queue.queue,
            .width = @intCast(size_x),
            .height = @intCast(size_y),
            .textureNum = 3,
            .format = .BT709_G22_8BIT,
            .flags = .{ .VSYNC = false },
            .queuedFrameNum = 2,
            .scaling = .ONE_TO_ONE,
            .gravityX = .MIN,
            .gravityY = .MIN,
        };
        return RenderContext.initNri(device, queue, swapchain_desc, arena, alloc);
    }

    fn initDevice(arena: std.mem.Allocator, instance_extensions: []const [*]u8) !nri.Device {
        // const khr_surface_name: [*c]u8 = @constCast("VK_KHR_surface");
        // const extensions_array = [_][*c]u8{khr_surface_name};
        const device_desc_create = nri.api.DeviceCreationDesc{
            .graphicsAPI = .{ .VK = true },
            .enableNRIValidation = true,
            .enableGraphicsAPIValidation = true,
            .vkExtensions = .{ .instanceExtensions = instance_extensions },
        };
        var device = try nri.Device.createDevice(device_desc_create, arena, &[_]nri.Interface{ nri.Interface.Core, nri.Interface.Swapchain, nri.Interface.Helper });

        const device_desc = device.getDeviceDesc();
        if (device_desc) |desc| {
            std.debug.print("nri version: {}\n", .{desc.nriVersion});
        }
        return device;
    }
    pub fn initNri(in_device: nri.Device, render_queue: nri.Queue, swapchain_desc: nri.api.SwapChainDesc, arena: std.mem.Allocator, alloc: std.mem.Allocator) !RenderContext {
        var device = in_device;
        const swapchain = try device.createSwapchain(swapchain_desc);
        const textures = swapchain.getTextures();
        const texture_desc = textures[0].getDesc(&device);
        const swapchain_views = try alloc.alloc(nri.Descriptor, textures.len);

        for (textures, 0..) |texture, i| {
            const texture_descriptor = try device.createTextureView(.{
                .texture = texture.texture,
                .type = nri.api.TextureView.COLOR_ATTACHMENT,
                .format = texture_desc.format,
                .mipOffset = 0,
                .mipNum = texture_desc.mipNum,
                .layerOffset = 0,
                .layerNum = texture_desc.layerNum,
                .sliceOffset = 0,
                .sliceNum = texture_desc.sampleNum,
                .planes = nri.api.PlaneBits{ .COLOR = true },
                .components = .{},
            });
            swapchain_views[i] = texture_descriptor;
        }
        return RenderContext{
            .device = device,
            .render_queue = render_queue,
            .swapchain = swapchain,
            .swapchain_textures = textures,
            .swapchain_texture_views = swapchain_views,
            .arena = arena,
            .alloc = alloc,
        };
    }
    fn initSurface() RenderContext {}
};

test {
    std.testing.refAllDecls(@This());
    std.testing.refAllDecls(RenderContext);
}
