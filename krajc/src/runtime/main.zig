const std = @import("std");
const krajc = @import("krajc");
const nri = @import("nri");
const sdl = @import("sdl");
pub const render = @import("render_context.zig");
const nri_c = nri.nri_c;
pub fn main(init: std.process.Init) !void {
    defer sdl.shutdown();
    const sdl_flags = sdl.InitFlags{ .events = true, .video = true };
    try sdl.init(sdl_flags);
    defer sdl.quit(sdl_flags);

    const window = try sdl.video.Window.init("Krajc engine", 1920, 1080, .{ .vulkan = true, .resizable = true, .transparent = true });
    var render_ctx = try render.RenderContext.fromSdl3(window, init.arena.allocator(), init.gpa);
    const device = &render_ctx.device;
    const queue = &render_ctx.render_queue;
    const swapchain = &render_ctx.swapchain;

    var aquire_fence = try device.createSwapchainFence();
    defer aquire_fence.destroy();
    var release_fence = try device.createSwapchainFence();
    defer release_fence.destroy();

    var frame_sync_fence = try device.createTimelineFence(0);
    defer frame_sync_fence.destroy();

    var command_alloc = try queue.createCommandAllocator();
    defer command_alloc.destroy();

    var command_buff = try command_alloc.createCommandBuffer();
    defer command_buff.destroy();

    var descriptor_pool = try device.createDescriptorPool(.{ .descriptorSetMaxNum = 1, .textureMaxNum = 1 });
    defer descriptor_pool.destory();

    var running = true;
    var frame_counter: u32 = 2;
    var frame_arena_alloc = std.heap.ArenaAllocator.init(init.gpa);
    const frame_arena = frame_arena_alloc.allocator();
    var frame_start_time = std.Io.Clock.real.now(init.io).toMicroseconds();
    while (running) {
        defer _ = frame_arena_alloc.reset(.retain_capacity);

        while (sdl.events.poll()) |event| {
            switch (event) {
                .quit => running = false,
                .key_down => |key| {
                    if (key.scancode.? == .q) {
                        running = false;
                    }
                },
                .window_resized => |window_resize| {
                    std.debug.print("resized: {}", .{window_resize});
                },
                else => {},
            }
        }
        command_alloc.reset();

        const texture_idx = try swapchain.aquireNextTexture(&aquire_fence);
        const curr_texture = render_ctx.swapchain_textures[texture_idx];
        const curr_texture_view = render_ctx.swapchain_texture_views[texture_idx];

        try command_buff.begin(&descriptor_pool);

        try command_buff.barrier(frame_arena, .{
            .textures = &[_]nri.api.TextureBarrierDesc{.{
                .texture = curr_texture.texture,
                .after = .{
                    .access = .{ .COLOR_ATTACHMENT_READ = true, .COLOR_ATTACHMENT_WRITE = true },
                    .layout = .COLOR_ATTACHMENT,
                },
            }},
        });
        try command_buff.beginRendering(frame_arena, .{
            .colors = &[_]nri.api.AttachmentDesc{.{
                .descriptor = curr_texture_view.descriptor,
                .loadOp = .CLEAR,
                .storeOp = .STORE,
                .clearValue = .{ .color = .{ .f = .{ .x = 0.1, .y = 0.2, .z = 0.3, .w = 0.1 } } },
            }},
        });
        command_buff.endRendering();
        try command_buff.barrier(frame_arena, .{
            .textures = &[_]nri.api.TextureBarrierDesc{.{
                .texture = curr_texture.texture,
                .before = .{
                    .access = .{ .COLOR_ATTACHMENT_READ = true, .COLOR_ATTACHMENT_WRITE = true },
                    .layout = .COLOR_ATTACHMENT,
                },
                .after = .{
                    .access = .{},
                    .stages = @bitCast(nri.api.StageBits.NONE),
                    .layout = .PRESENT,
                },
            }},
        });

        try command_buff.end();
        try queue.submit(frame_arena, .{
            .commandBuffers = &[_]*nri_c.CommandBuffer{command_buff.command_buffer},
            .waitFences = &[_]nri.api.FenceSubmitDesc{nri.api.FenceSubmitDesc{ .fence = aquire_fence.fence, .value = 0, .stages = .{ .COLOR_ATTACHMENT = true } }},
            .signalFences = &[_]nri.api.FenceSubmitDesc{
                nri.api.FenceSubmitDesc{ .fence = release_fence.fence, .value = 0, .stages = .{ .COLOR_ATTACHMENT = true } },
                nri.api.FenceSubmitDesc{ .fence = frame_sync_fence.fence, .value = frame_counter, .stages = @bitCast(nri.api.StageBits.ALL) },
            },
            .swapChain = swapchain.swapchain,
        });
        try swapchain.queuePresent(&release_fence);
        frame_sync_fence.wait(frame_counter);
        frame_counter += 1;
        const now = std.Io.Clock.real.now(init.io).toMicroseconds();
        const frame_dt = now - frame_start_time;
        frame_start_time = now;
        std.debug.print("frame: {}, dt: {}\n", .{ frame_counter, @as(f32, @floatFromInt(frame_dt)) / @as(f32, @floatFromInt(std.time.us_per_ms)) });
    }
}

test {
    std.testing.refAllDecls(@This());
    std.testing.refAllDecls(render);
}
//
//
//
//
//
