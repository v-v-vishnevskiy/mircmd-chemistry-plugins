struct Uniforms {
    view_projection: mat4x4<f32>,
    scene_transform: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
};

struct InstanceInput {
    @location(2) model_matrix_0: vec4<f32>,
    @location(3) model_matrix_1: vec4<f32>,
    @location(4) model_matrix_2: vec4<f32>,
    @location(5) model_matrix_3: vec4<f32>,
    @location(6) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) normal: vec3<f32>,
    @location(1) color: vec4<f32>,
};

@vertex
fn vs_main(vertex: VertexInput, instance: InstanceInput) -> VertexOutput {
    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );

    var output: VertexOutput;
    let world_position = model_matrix * vec4<f32>(vertex.position, 1.0);
    output.position = uniforms.view_projection * world_position;
    output.normal = (uniforms.scene_transform * model_matrix * vec4<f32>(vertex.normal, 0.0)).xyz;
    output.color = instance.color;
    return output;
}

fn calculate_blinn_phong(
    fragment_color: vec4<f32>,
    normal: vec3<f32>,
    light_color: vec3<f32>,
    light_position: vec3<f32>,
    ambient_strength: f32,
    specular_strength: f32,
    shininess: f32
) -> vec4<f32> {
    // Ambient
    let ambient: vec3<f32> = light_color * ambient_strength;

    // Diffuse
    let light_direction = normalize(light_position);
    let diff: f32 = max(dot(normalize(normal), light_direction), 0.0);
    let diffuse: vec3<f32> = diff * light_color;

    // Specular
    let pi: f32 = 3.14159265;
    let energy_conservation: f32 = ( 8.0 + shininess ) / ( 8.0 * pi );
    let spec: f32 = energy_conservation * pow(diff, shininess);
    let specular: vec3<f32> = specular_strength * spec * light_color;

    // Final
    let final_color: vec3<f32> = ((ambient + diffuse) * fragment_color.xyz) + specular;
    return vec4<f32>(final_color, fragment_color.w);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let ambient_strength: f32 = 0.3;
    let specular_strength: f32 = 0.6;
    let shininess: f32 = 16.0;
    let light_color = vec3<f32>(1.0, 1.0, 1.0) * 0.9;
    let light_position = vec3<f32>(0.3, 0.3, 1.0);

    return calculate_blinn_phong(
        in.color, 
        in.normal,
        light_color, 
        light_position, 
        ambient_strength, 
        specular_strength, 
        shininess
    );
}
