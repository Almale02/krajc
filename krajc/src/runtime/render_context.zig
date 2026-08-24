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
    swapchain_arena: std.heap.ArenaAllocator,
    alloc: std.mem.Allocator,

    pub fn fromSdl3(window: *sdl.video.Window, arena: std.mem.Allocator, alloc: std.mem.Allocator) !RenderContext {
        const extensions_null = try sdl.vulkan.getInstanceExtensions();
        var extensions = try arena.alloc([*]u8, extensions_null.len);
        for (0..extensions_null.len) |i| {
            extensions[i] = @ptrCast(@constCast(extensions_null[i]));
        }

        var device = try RenderContext.initDevice(arena, extensions);
        const queue = try device.getQueue(.GRAPHICS, 0);

        var render_ctx = try RenderContext.initNri(device, queue, arena, alloc);
        try render_ctx.createSwapchain(window, true);
        return render_ctx;
    }

    fn initDevice(arena: std.mem.Allocator, instance_extensions: []const [*]u8) !nri.Device {
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
    fn initNri(
        device: nri.Device,
        render_queue: nri.Queue,
        arena: std.mem.Allocator,
        alloc: std.mem.Allocator,
    ) !RenderContext {
        return RenderContext{
            .device = device,
            .render_queue = render_queue,
            .swapchain = undefined,
            .swapchain_textures = undefined,
            .swapchain_texture_views = undefined,
            .arena = arena,
            .swapchain_arena = std.heap.ArenaAllocator.init(alloc),
            .alloc = alloc,
        };
    }
    pub fn createSwapchain(self: *RenderContext, window: *sdl.video.Window, init: bool) !void {
        const props = try window.getProperties();
        const wayland_surface = props.wayland_surface.?.value;
        const wayland_display = props.wayland_display.?.value;
        const size = try window.getSizeInPixels();
        const size_x = size.@"0";
        const size_y = size.@"1";

        const swapchain_desc = nri.api.SwapChainDesc{
            .window = .{ .wayland = .{ .display = wayland_display, .surface = wayland_surface } },
            .queue = self.render_queue.queue,
            .width = @intCast(size_x),
            .height = @intCast(size_y),
            .textureNum = 3,
            .format = .BT709_G22_8BIT,
            .flags = .{ .VSYNC = true },
            .queuedFrameNum = 2,
            .scaling = .ONE_TO_ONE,
            .gravityX = .MIN,
            .gravityY = .MIN,
        };
        if (!init) {
            try self.render_queue.waitIdle();
            self.swapchain.destroy();
            for (self.swapchain_texture_views) |view| {
                view.destroy(&self.device);
            }
        }
        _ = self.swapchain_arena.reset(.retain_capacity);
        const swapchain_arena = self.swapchain_arena.allocator();

        const swapchain = try self.device.createSwapchain(swapchain_desc);
        const textures = swapchain.getTextures();
        const texture_desc = textures[0].getDesc(&self.device);
        const swapchain_views = try swapchain_arena.alloc(nri.Descriptor, textures.len);

        std.debug.print("created window: {}x{}\n", .{ texture_desc.width, texture_desc.height });

        for (textures, 0..) |texture, i| {
            const texture_descriptor = try self.device.createTextureView(.{
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
        self.swapchain = swapchain;
        self.swapchain_textures = textures;
        self.swapchain_texture_views = swapchain_views;
    }
    pub fn deinit(self: *RenderContext) void {
        self.swapchain_arena.deinit();
    }
};

test {
    std.testing.refAllDecls(@This());
    std.testing.refAllDecls(RenderContext);
}
