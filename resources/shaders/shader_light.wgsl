// Vertex shader

struct CameraUniform {
    view_proj: mat4x4<f32>,
    view_pos: vec4<f32>,
};

struct Light {
    position: vec4<f32>,
    color: vec4<f32>,
    rot: vec4<f32>,
}

struct InstanceInput {
    @location(5) model_matrix_0: vec4<f32>,
    @location(6) model_matrix_1: vec4<f32>,
    @location(7) model_matrix_2: vec4<f32>,
    @location(8) model_matrix_3: vec4<f32>,
};

@group(1) @binding(0) // 1.
var<uniform> camera: CameraUniform;

@group(2) @binding(0)
var<uniform> light: Light;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) normal: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) world_position: vec3<f32>,
    @location(3) camera_pos: vec3<f32>,
};

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );
    // NEW!
    var out: VertexOutput;
    out.uv = model.uv;

    out.world_normal = normalize((model_matrix * vec4<f32>(model.normal, 0.0)).xyz);
    var world_position: vec4<f32> = model_matrix * vec4<f32>(model.position, 1.0);
    out.world_position = world_position.xyz;
    out.clip_position = camera.view_proj * world_position;
    out.camera_pos = camera.view_pos.xyz;
    return out;
}
 
// Fragment shader



@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let obj_color = textureSample(t_diffuse, s_diffuse, in.uv);
    
    //let light_color = vec3<f32>(1.0, 1.0, 1.0);
    //let light_color = light.color.xyz;
    

    //let ambient_strenght = 0.08;
    //let ambient_color = light_color * ambient_strenght;

    
    //let light_dir = normalize(light.position.xyz - in.world_position);

    //let diffuse_strength_mod = 1.0;
    //let diffuse_strength = max(dot(in.world_normal, light_dir), 0.0) * diffuse_strength_mod;
    //let diffuse_color = light_color * diffuse_strength;
    

    ////let view_dir = normalize(camera.view_pos.xyz - in.world_position);
    ////let reflect_dir = reflect(-light_dir, in.world_normal);



    //let camera_pos = in.camera_pos;
    //let view_pos = normalize(camera_pos);
    //let reflect_source = normalize(reflect(-light.position.xyz - in.world_position, normalize(in.world_normal)));
    
    
    //let view_dir = normalize(camera.view_pos.xyz - in.world_position);
    //let half_dir = normalize(view_dir + light_dir);

    
    //let specular_strength_mod = 4.8;
    //let specular_strength = pow(max(dot(view_dir, reflect_dir), 0.), 32.) * specular_strength_mod;
    //let specular_strength = pow(max(dot(normalize(view_pos - in.world_position), reflect_source), 0.0), 32.0) * specular_strength_mod;
    //let specular_color = specular_strength * light_color;

    
    //let result = obj_color.xyz * (ambient_color + specular_color);// * (ambient_color * diffuse_color);
    //let resulta = specular_color;

    //return vec4<f32>(result, obj_color.a);



 // Normalize vectors
    let normal = normalize(in.world_normal);
    let light_dir = normalize(light.position.xyz - in.world_position);
    let view_dir = normalize(in.camera_pos - in.world_position);
    let reflect_dir = reflect(-light_dir, normal);

    // Ambient
    let ambient_strenght = 0.08;
    let ambient = light.color.rgb * ambient_strenght;

    // Diffuse
    let diff = max(dot(normal, light_dir), 0.0);
    let diffuse = diff * light.color.rgb;

    // Specular
    let spec = pow(max(dot(view_dir, reflect_dir), 0.0), 64.0) * 5.;
    let specular = spec * light.color.rgb;

    // Combine all components
    return vec4<f32>(obj_color.xyz * (specular + diffuse + ambient), 1.0);    
}
