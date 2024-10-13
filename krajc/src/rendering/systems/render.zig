const std = @import("std");
const wgpu = @import("wgpu");
const krajc = @import("../../prelude.zig");

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

    render_pass.setPipeline(state.render_pipeline);
    render_pass.draw(3, 1, 0, 0);

    render_pass.end();
    render_pass.release();

    const command_buffer = encoder.finish(null);
    encoder.release();

    state.queue.submit(&[_]*wgpu.CommandBuffer{command_buffer});
    command_buffer.release();
    state.surface_swap_chain.present();
    surface_view.release();
}
