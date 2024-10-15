const math = @import("../../math.zig");
const std = @import("std");
const wgpu = @import("wgpu");
const krajc = @import("../../prelude.zig");

pub const Vertex = struct {
    pos: [3]f32,
    p1: f32 = 0.0,
    color: [3]f32,
    p2: f32 = 0.0,

    pub fn new(pos: math.Vec3, color: math.Vec3) Vertex {
        return Vertex{
            .pos = [_]f32{ pos.x(), pos.y(), pos.z() },
            .color = [_]f32{ color.x(), color.y(), color.z() },
        };
    }
};

pub fn render(res: *krajc.ResourceState) !void {
    const state = res.get(krajc.RenderingState);

    const surface_view = state.surface_swap_chain.getCurrentTextureView().?;

    const color_attachment = wgpu.RenderPassColorAttachment{
        .view = surface_view,
        .resolve_target = null,
        .clear_value = std.mem.zeroes(wgpu.Color),
        .load_op = .clear,
        .store_op = .store,
    };
    const encoder = state.device.createCommandEncoder(&.{ .label = "Command Encoder" });

    const render_pass_info = wgpu.RenderPassDescriptor.init(.{
        .color_attachments = &.{color_attachment},
    });
    var render_pass = encoder.beginRenderPass(&render_pass_info);

    const vertex_buffer = state.device.createBuffer(&.{ .usage = .{ .vertex = false, .copy_dst = false }, .size = 512 * 512 });
    defer vertex_buffer.destroy();
    const vertex_data = [_]f32{
        //Vertex.new(math.Vec3.new(-1.0, -1.0, 0.0), math.Vec3.new(1.0, 1.0, 1.0)), // Bottom-left vertex
        //Vertex.new(math.Vec3.new(1.0, -1.0, 0.0), math.Vec3.new(1.0, 1.0, 1.0)), // Bottom-right vertex
        //Vertex.new(math.Vec3.new(0.0, 1.0, 0.0), math.Vec3.new(1.0, 1.0, 1.0)), // Top vertex
        1.0,
        2.0,
        3.0,
    };
    const slice = vertex_data[0..];
    state.queue.writeBuffer(
        vertex_buffer,
        0,
        slice,
    );

    render_pass.setPipeline(state.render_pipeline);
    render_pass.setVertexBuffer(0, vertex_buffer, 0, 4 * 3);
    render_pass.draw(3, 1, 0, 0);

    render_pass.end();
    render_pass.release();

    const command_buffer = encoder.finish(null);
    encoder.release();
    state.instance.processEvents();

    state.queue.submit(&[_]*wgpu.CommandBuffer{command_buffer});
    command_buffer.release();
    state.surface_swap_chain.present();
    surface_view.release();
    state.instance.processEvents();
}
