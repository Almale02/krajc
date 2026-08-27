const nri = @import("nri");
const std = @import("std");
const sdl = @import("sdl");

pub const Vec3 = struct {
    x: f32,
    y: f32,
    z: f32,

    pub fn new(x: f32, y: f32, z: f32) Vec3 {
        return Vec3{ .x = x, .y = y, .z = z };
    }
};
pub const Vertex = struct {
    pos: Vec3,
    color: Vec3,
};
pub const PushConstant = struct {
    vertex_buffer_addr: usize,
};

pub const RenderContext = struct {
    device: nri.Device,
    render_queue: nri.Queue,
    swapchain: nri.Swapchain,
    swapchain_textures: []const nri.Texture,
    swapchain_texture_views: []const nri.Descriptor,
    arena: std.mem.Allocator,
    swapchain_arena: std.heap.ArenaAllocator,
    alloc: std.mem.Allocator,

    vertex_shader: []u8,
    fragment_shader: []u8,

    mesh: []Vertex,
    mesh_mem: nri.Memory,
    mesh_buffer: nri.Buffer,

    pub fn fromSdl3(window: *sdl.video.Window, arena: std.mem.Allocator, alloc: std.mem.Allocator, io: std.Io) !RenderContext {
        const extensions_null = try sdl.vulkan.getInstanceExtensions();
        var extensions = try arena.alloc([*]u8, extensions_null.len);
        for (0..extensions_null.len) |i| {
            extensions[i] = @ptrCast(@constCast(extensions_null[i]));
        }

        var device = try RenderContext.initDevice(arena, extensions);
        const queue = try device.getQueue(.GRAPHICS, 0);

        var render_ctx = try RenderContext.initNri(device, queue, arena, alloc);
        try render_ctx.createSwapchain(window, true);
        try render_ctx.loadShaders(io);
        try render_ctx.create_mesh();

        const mesh_buffer_size = @sizeOf(Vertex) * render_ctx.mesh.len;
        const mesh_buffer_desc = nri.api.BufferDesc{
            .size = mesh_buffer_size,
            .usage = .{ .SHADER_RESOURCE_STORAGE = true },
        };
        render_ctx.mesh_buffer = try device.createBuffer(mesh_buffer_desc);
        const mem_desc = try device.getBufferMemoryDesc(mesh_buffer_desc, .DEVICE_UPLOAD);
        render_ctx.mesh_mem = try device.allocateMemory(nri.api.AllocateMemoryDesc{
            .size = mem_desc.size,
            .type = mem_desc.type,
            .priority = 0.6,
            .vma = .{ .enable = true, .alignment = mem_desc.alignment },
        });
        try device.bindBufferMemory(arena, &[_]nri.api.BindBufferMemoryDesc{.{
            .buffer = render_ctx.mesh_buffer.buffer,
            .memory = render_ctx.mesh_mem,
            .offset = 0,
        }});
        const mesh_buffer_ptr = try render_ctx.mesh_buffer.map(&device, 0, mesh_buffer_size);
        @memcpy(@as([*]Vertex, @ptrCast(@alignCast(mesh_buffer_ptr.ptr)))[0..render_ctx.mesh.len], render_ctx.mesh);
        render_ctx.mesh_buffer.unmap(&device);
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
            .vertex_shader = undefined,
            .fragment_shader = undefined,
            .mesh = undefined,
            .mesh_mem = undefined,
            .mesh_buffer = undefined,
        };
    }
    pub fn createSwapchain(self: *RenderContext, window: *sdl.video.Window, init: bool) !void {
        sdl.events.pump();
        const props = try window.getProperties();
        const size = try window.getSizeInPixels();
        const size_x = size.@"0";
        const size_y = size.@"1";

        var window_handle: ?nri.api.Window = null;

        if (std.mem.eql(u8, sdl.video.getCurrentDriverName().?, "wayland")) {
            window_handle = nri.api.Window{ .wayland = .{ .display = props.wayland_display.?.value, .surface = props.wayland_surface.?.value } };
        }
        if (std.mem.eql(u8, sdl.video.getCurrentDriverName().?, "x11")) {
            window_handle = nri.api.Window{ .x11 = .{ .window = @intCast(props.x11_window.?), .dpy = props.x11_display.?.value } };
        }

        const swapchain_desc = nri.api.SwapChainDesc{
            .window = window_handle orelse @panic("could not init swapchain"),
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
    pub fn loadShaders(self: *RenderContext, io: std.Io) !void {
        const shader_dir = try std.Io.Dir.cwd().openDir(io, "assets/shaders/out/vulkan", .{});
        const vertex_file = try shader_dir.openFile(io, "triangle_vert.spv", .{});
        const vertex_stat = try vertex_file.stat(io);
        self.vertex_shader = try self.arena.alloc(u8, vertex_stat.size);
        _ = try vertex_file.readPositionalAll(io, self.vertex_shader, 0);

        const fragment_file = try shader_dir.openFile(io, "triangle_frag.spv", .{});
        const fragment_stat = try fragment_file.stat(io);
        self.fragment_shader = try self.arena.alloc(u8, fragment_stat.size);
        _ = try fragment_file.readPositionalAll(io, self.fragment_shader, 0);
    }
    pub fn create_mesh(self: *RenderContext) !void {
        self.mesh = try self.arena.alloc(Vertex, 3);
        self.mesh[0] = Vertex{ .pos = Vec3.new(-0.5, 0.5, 0), .color = Vec3.new(0.7, 0.1, 0.1) };
        self.mesh[1] = Vertex{ .pos = Vec3.new(0.5, 0.5, 0), .color = Vec3.new(0.5, 0.3, 1) };
        self.mesh[2] = Vertex{ .pos = Vec3.new(0, -0.5, 0), .color = Vec3.new(0.2, 0.5, 0.0) };
    }
    pub fn deinit(self: *RenderContext) void {
        self.mesh_buffer.destroy(&self.device);
        self.device.freeMemory(self.mesh_mem);
        for (self.swapchain_texture_views) |view| view.destroy(&self.device);
        self.swapchain.destroy();
        self.device.destroy();

        self.swapchain_arena.deinit();
    }
};

test {
    std.testing.refAllDecls(@This());
    std.testing.refAllDecls(RenderContext);
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
