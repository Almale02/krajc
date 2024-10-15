const std = @import("std");
const utils = @import("../../utils.zig");
const glfw = @import("glfw");
const wgpu = @import("wgpu");
const math = @import("../../math.zig");
const main = @import("../../main.zig");
const krajc = @import("../../prelude.zig");

const Vertex = @import("../systems/render.zig").Vertex;

const c = @cImport({
    @cInclude("dawn/webgpu.h");
    @cInclude("mach_dawn.h");
});

pub const RenderingState = struct {
    window: glfw.Window = undefined,
    window_surface: *wgpu.Surface = undefined,
    surface_swap_chain: *wgpu.SwapChain = undefined,
    device: *wgpu.Device = undefined,
    adapter: *wgpu.Adapter = undefined,
    queue: *wgpu.Queue = undefined,
    render_pipeline: *wgpu.RenderPipeline = undefined,
    instance: *wgpu.Instance = undefined,
};
inline fn err_callback(err: wgpu.ErrorType, msg: [*:0]const u8) void {
    _ = err;
    std.debug.panic("wgpu error: {s}", .{msg});
}
inline fn log_callback(err: wgpu.LoggingType, msg: [*:0]const u8) void {
    _ = err;
    std.debug.panic("wgpu log: {s}", .{msg});
}

pub fn init_wgpu(window: glfw.Window, render_state: *RenderingState, asset_manager: *krajc.AssetManager) !void {
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
    device.pushErrorScope(.validation);
    device.injectError(.validation, "test error");
    //c.wgpuDeviceSetUncapturedErrorCallback(device.createErrorBuffer(), , );

    device.setUncapturedErrorCallback(utils.printUnhandledErrorCallback);
    device.pushErrorScope(.out_of_memory);
    device.pushErrorScope(.internal);
    device.pushErrorScope(.validation);
    device.setLoggingCallback(log_callback);
    // const bind_group_layouts: []const *wgpu.BindGroupLayout = [_]*wgpu.BindGroupLayout{
    //     //device.createBindGroupLayout(*wgpu.BindGroupLayout.Descriptor.init(null, null, wgpu.BindGroupLayout.Entry.buffer(0, .{ .vertex = true, .fragment = true }, .uniform, false, 512 * 512))),
    // };
    const desc = wgpu.PipelineLayout.Descriptor.init(.{ .label = "pipeline layout" });
    const pipeline_layout = device.createPipelineLayout(&desc);

    const shader_asset = krajc.ShaderLoader.start(asset_manager, device, "src/shader.wgsl");

    while (!shader_asset.handle.is_loaded()) {
        std.time.sleep(std.time.ns_per_ms * 80);
    }
    const shader_module = shader_asset.handle.get();
    instance.processEvents();

    // const vs =
    //     \\ @vertex fn main(
    //     \\     @builtin(vertex_index) VertexIndex : u32
    //     \\ ) -> @builtin(position) vec4<f32> {
    //     \\     var pos = array<vec2<f32>, 3>(
    //     \\         vec2<f32>( 0.0,  0.5),
    //     \\         vec2<f32>(-0.5, -0.5),
    //     \\         vec2<f32>( 0.5, -0.5)
    //     \\     );
    //     \\     return vec4<f32>(pos[VertexIndex], 0.0, 1.0);
    //     \\ }
    // ;
    // const vs_module = device.createShaderModuleWGSL("my vertex shader", vs);

    // const fs =
    //     \\ @fragment fn main() -> @location(0) vec4<f32> {
    //     \\     return vec4<f32>(1.0, 0.0, 0.0, 1.0);
    //     \\ }
    // ;
    // const fs_module = device.createShaderModuleWGSL("my fragment shader", fs);

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
        .module = shader_module,
        .entry_point = "fs_main",
        .targets = &.{color_target},
    });
    const vertex_buffers = [_]wgpu.VertexBufferLayout{
        wgpu.VertexBufferLayout.init(.{ .array_stride = 4, .step_mode = wgpu.VertexStepMode.vertex }),
    };
    const buffers_slice = vertex_buffers[0..];
    const vertex = wgpu.VertexState{ .module = shader_module, .entry_point = "vs_main", .buffer_count = 1, .buffers = buffers_slice.ptr };

    const pipeline = device.createRenderPipeline(&wgpu.RenderPipeline.Descriptor{
        .layout = pipeline_layout,
        .depth_stencil = null,
        .vertex = vertex,
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
    render_state.instance = instance;
}
