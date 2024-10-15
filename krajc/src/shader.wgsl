
// Vertex shader

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) p1: f32,
    @location(2) color: vec3<f32>,
    @location(3) p2: f32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
};

fn vs_maina(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    //out.color = model.color;
    out.clip_position = vec4<f32>(model.position, 1.0);
    return out;
}

@vertex
fn vs_main(
    @location(0) model: f32,
    @builtin(vertex_index) VertexIndex : u32
) -> VertexOutput {
    var out: VertexOutput;

    var pos = array<vec2<f32>, 3>(
        vec2<f32>( 0.0,  0.5),
        vec2<f32>(-0.5, -0.5),
        vec2<f32>( 0.5, -0.5)
    );
    
    out.clip_position = vec4<f32>(pos[VertexIndex], 0.0, 1.0);
    return out;
}


// Fragment shader

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 0.0, 1.0, 1.0);
}

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
