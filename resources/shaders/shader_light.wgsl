// Vertex shader

struct PointLightUniform {
    position: vec4<f32>,
    color: vec4<f32>,
    light_strenght: f32,
    ambient_strenght: f32,
    _p1: f32,
    _p2: f32,
}
    
struct SpotLightUniform {
    position: vec4<f32>,
    color: vec4<f32>,
    direction: vec4<f32>,
    light_strenght: f32,
    ambient_strenght: f32,
    inner_angle: f32,
    outer_angle: f32,
    _padding: vec4<f32>,
}


struct CameraUniform {
    view_proj: mat4x4<f32>,
    view_pos: vec4<f32>,
};

struct Light {
    position: vec4<f32>,
    color: vec4<f32>,
    rot: vec4<f32>,
}

struct Index {
    index: u32,
    _p1: u32,
    _p2: u32,
    _p3: u32,
}
struct InstanceInput {
    @location(5) model_matrix_0: vec4<f32>,
    @location(6) model_matrix_1: vec4<f32>,
    @location(7) model_matrix_2: vec4<f32>,
    @location(8) model_matrix_3: vec4<f32>,
};

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@group(1) @binding(0)
var<uniform> camera: CameraUniform;

@group(2) @binding(0)
var<storage, read> point_light_array: array<PointLightUniform>;
@group(2) @binding(1) 
var<uniform> point_light_count: u32;

@group(3) @binding(0)
var<storage, read> spot_light_array: array<SpotLightUniform>;
@group(3) @binding(1) 
var<uniform> spot_light_count: u32;

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
    var out: VertexOutput;
    out.uv = model.uv;

    out.world_normal = normalize((model_matrix * vec4<f32>(model.normal, 0.0)).xyz);
    var world_position: vec4<f32> = model_matrix * vec4<f32>(model.position, 1.0);
    out.world_position = world_position.xyz;
    out.clip_position = camera.view_proj * world_position;
    out.camera_pos = camera.view_pos.xyz;
    return out;
}
 



// here is the code for the point light
//fn asd(in: VertexOutput) -> @location(0) vec4<f32> {
    //let obj_color = textureSample(t_diffuse, s_diffuse, in.uv);
    
    //let light_color = light.color.xyz;
    

    //let ambient_strenght = 0.08;
    //let ambient_color = light_color * ambient_strenght;

    
    //let light_dir = normalize(light.position.xyz - in.world_position);

    //let diffuse_strength_mod = 1.0;
    //let diffuse_strength = max(dot(in.world_normal, light_dir), 0.0) * diffuse_strength_mod;
    //let diffuse_color = light_color * diffuse_strength;
    

    //let camera_pos = in.camera_pos;
    //let view_pos = normalize(camera_pos);
    
    
    //let view_dir = normalize(camera.view_pos.xyz - in.world_position);
    //let half_dir = normalize(view_dir + light_dir);

    
    //let specular_strength_mod = 4.8;
    //let specular_strength = pow(max(dot(in.world_normal, half_dir), 0.), 32.) * specular_strength_mod;
    //let specular_color = specular_strength * light_color;

    
    //let result = obj_color.xyz * (ambient_color + diffuse_color + specular_color);
    //let resulta = specular_color;

    //return vec4<f32>(result, obj_color.a);

//}


// HERE is the code for the spot light
//@fragment
//fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    //let obj_color = textureSample(t_diffuse, s_diffuse, in.uv);
//
    //let light_color = light.color.xyz;
    //var light_pos = light.position.xyz;
    //let light_dir = normalize(light_pos - in.camera_pos);
//
    //// Calculate the direction from the fragment position to the light source
    //let light_to_frag = light_pos - in.world_position;
    //let light_distance = length(light_to_frag);
    //let light_dir_from_frag = normalize(light_to_frag);
//
    //// Calculate the angle between the light direction and the direction to the fragment
    //let light_angle = dot(light_dir_from_frag, light_dir);
//
    //// Spotlight cone angles
    //let spotlight_inner_angle = cos(0.5); // Example value, adjust as needed
    //let spotlight_outer_angle = cos(0.7); // Example value, adjust as needed
//
    //// Attenuation factor based on the spotlight cone
    //let spotlight_attenuation = clamp((light_angle - spotlight_outer_angle) / (spotlight_inner_angle - spotlight_outer_angle), 0.0, 1.0);
//
    //// Ambient lighting
    //let ambient_strength = 0.08;
    //let ambient_color = ambient_strength;
//
    //// Diffuse lighting
    //let diffuse_strength = max(dot(in.world_normal, light_dir_from_frag), 0.0);
    //let diffuse_color = light_color * diffuse_strength * spotlight_attenuation;
//
    //// Specular lighting
    //let view_dir = normalize(in.camera_pos - in.world_position);
    //let half_dir = normalize(view_dir + light_dir_from_frag);
    //let specular_strength = pow(max(dot(in.world_normal, half_dir), 0.0), 32.0);
    //let specular_color = specular_strength * light_color * spotlight_attenuation;
//
    //// Combine all components
    //let result = obj_color.xyz * (ambient_color + diffuse_color + specular_color);
//
    //return vec4<f32>(result, obj_color.a);
//}



@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let obj_color = textureSample(t_diffuse, s_diffuse, in.uv);

    // Initialize the lighting result
    var final_color = vec3<f32>(0.0, 0.0, 0.0);
    // Handle Point Lights
    for (var i = 0u; i < point_light_count; i = i + 1u) {
        let light = point_light_array[i];
        let light_dir = normalize(light.position.xyz - in.world_position);
        
        // Ambient lighting
        let ambient_strength = light.ambient_strenght;
        let light_color = light.color.xyz;
        let ambient_color = light_color.xyz * ambient_strength;

        // Diffuse lighting
        let diffuse_strength = max(dot(in.world_normal, light_dir), 0.0);
        let diffuse_color = light_color * diffuse_strength;

        // Specular lighting
        let view_dir = normalize(camera.view_pos.xyz - in.world_position);
        let half_dir = normalize(view_dir + light_dir);
        let specular_strength = pow(max(dot(in.world_normal, half_dir), 0.0), 32.0);
        let specular_color = specular_strength * light.color.xyz;

        // Add the contribution from this point light
        final_color = final_color + (obj_color.xyz * (ambient_color + light.light_strenght * (diffuse_color + specular_color)));
        //final_color = vec4<f32>(1., 1., 1., 1.).xyz;
    }
    // Handle Spot Lights
    for (var i = 0u; i < spot_light_count; i = i + 1u) {
        let light = spot_light_array[i];
        //let light_dir = normalize(light.position.xyz - in.camera_pos);
        let light_dir = normalize(light.direction.xyz);
        let light_to_frag = light.position.xyz - in.world_position;
        let light_distance = length(light_to_frag);
        let light_dir_from_frag = normalize(light_to_frag);

        // Calculate the angle between the light direction and the direction to the fragment
        let light_angle = dot(light_dir_from_frag, light_dir);

        // Spotlight cone angles (adjust these as needed)
        let spotlight_inner_angle = cos(light.inner_angle);
        let spotlight_outer_angle = cos(light.outer_angle);

        // Attenuation factor based on the spotlight cone
        let spotlight_attenuation = clamp((light_angle - spotlight_outer_angle) / (spotlight_inner_angle - spotlight_outer_angle), 0.0, 1.0);

        // Ambient lighting
        let ambient_strength = light.ambient_strenght;
        let ambient_color = light.color.xyz * ambient_strength;

        // Diffuse lighting
        let diffuse_strength = max(dot(in.world_normal, light_dir_from_frag), 0.0);
        let diffuse_color = light.color.xyz * diffuse_strength * spotlight_attenuation;

        // Specular lighting
        let view_dir = normalize(camera.view_pos.xyz - in.world_position);
        let half_dir = normalize(view_dir + light_dir_from_frag);
        let specular_strength = pow(max(dot(in.world_normal, half_dir), 0.0), 32.0);
        let specular_color = specular_strength * 0. * light.color.xyz * spotlight_attenuation;

        // Add the contribution from this spot light
        final_color = final_color +  (obj_color.xyz * (ambient_color + light.light_strenght * (diffuse_color + specular_color)));
    }



    // Return the final color with the original object's alpha value
    return vec4<f32>(final_color, obj_color.a);
}


