const std = @import("std");
const utils = @import("../../utils.zig");
const glfw = @import("glfw");
const wgpu = @import("wgpu");
const math = @import("../../math.zig");
const main = @import("../../main.zig");

pub const RenderingState = struct {
    window: glfw.Window = undefined,
    window_surface: *wgpu.Surface = undefined,
    surface_swap_chain: *wgpu.SwapChain = undefined,
    device: *wgpu.Device = undefined,
    adapter: *wgpu.Adapter = undefined,
    queue: *wgpu.Queue = undefined,
    render_pipeline: *wgpu.RenderPipeline = undefined,
};

pub fn init_wgpu(window: glfw.Window, render_state: *RenderingState) !void {
    const instance = wgpu.createInstance(&.{}).?;

    const surface = try utils.createSurfaceForWindow(instance, window, comptime utils.detectGLFWOptions());

    var adapter_response: utils.RequestAdapterResponse = undefined;
    instance.requestAdapter(&wgpu.RequestAdapterOptions{ .compatible_surface = surface, .power_preference = .high_performance, .force_fallback_adapter = .false }, &adapter_response, utils.requestAdapterCallback);
    var adapter: *wgpu.Adapter = adapter_response.adapter orelse {
        std.debug.panic("failed to create adapter", .{});
    };
    const device: *wgpu.Device = adapter.createDevice(null) orelse {
        std.debug.panic("failed to create device", .{});
    };
    // const bind_group_layouts: []const *wgpu.BindGroupLayout = [_]*wgpu.BindGroupLayout{
    //     //device.createBindGroupLayout(*wgpu.BindGroupLayout.Descriptor.init(null, null, wgpu.BindGroupLayout.Entry.buffer(0, .{ .vertex = true, .fragment = true }, .uniform, false, 512 * 512))),
    // };
    const desc = wgpu.PipelineLayout.Descriptor.init(.{ .label = "pipeline layout" });
    const pipeline_layout = device.createPipelineLayout(&desc);

    const vs =
        \\ @vertex fn main(
        \\     @builtin(vertex_index) VertexIndex : u32
        \\ ) -> @builtin(position) vec4<f32> {
        \\     var pos = array<vec2<f32>, 3>(
        \\         vec2<f32>( 0.0,  0.5),
        \\         vec2<f32>(-0.5, -0.5),
        \\         vec2<f32>( 0.5, -0.5)
        \\     );
        \\     return vec4<f32>(pos[VertexIndex], 0.0, 1.0);
        \\ }
    ;
    const vs_module = device.createShaderModuleWGSL("my vertex shader", vs);

    const fs =
        \\ @fragment fn main() -> @location(0) vec4<f32> {
        \\     return vec4<f32>(1.0, 0.0, 0.0, 1.0);
        \\ }
    ;
    const fs_module = device.createShaderModuleWGSL("my fragment shader", fs);

    // Fragment state
    const blend = wgpu.BlendState{
        .color = .{
            .dst_factor = .one,
        },
        .alpha = .{
            .dst_factor = .one,
        },
    };
    const color_target = wgpu.ColorTargetState{
        .format = .bgra8_unorm,
        .blend = &blend,
        .write_mask = wgpu.ColorWriteMaskFlags.all,
    };
    const fragment = wgpu.FragmentState.init(.{
        .module = fs_module,
        .entry_point = "main",
        .targets = &.{color_target},
    });

    const pipeline = device.createRenderPipeline(&wgpu.RenderPipeline.Descriptor{
        .layout = pipeline_layout,
        .depth_stencil = null,
        .vertex = wgpu.VertexState{ .module = vs_module, .entry_point = "main" },
        .fragment = &fragment,
        .multisample = .{ .count = 1 },
        .primitive = .{},
    });
    const queue = device.getQueue();
    const swap_chain = device.createSwapChain(surface, &.{ .width = 700, .height = 500, .usage = .{ .render_attachment = true }, .format = .bgra8_unorm, .present_mode = .mailbox });
    //_ = bind_group_layouts;
    //const pipeline_desc = wgpu.RenderPipeline.Descriptor {.label = "Pipeline Desc", .};
    render_state.window_surface = surface;
    render_state.device = device;
    render_state.adapter = adapter;
    render_state.render_pipeline = pipeline;
    render_state.queue = queue;
    render_state.surface_swap_chain = swap_chain;
}
