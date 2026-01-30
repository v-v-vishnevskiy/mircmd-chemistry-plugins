struct Uniforms {
    projection_transform: mat4x4<f32>,
    view_transform: mat4x4<f32>,
    scene_transform: mat4x4<f32>,
    final_transform: mat4x4<f32>, // projection_transform * view_transform * scene_transform
    is_perspective: u32,          // 1 = perspective, 0 = orthographic
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
    @location(7) ray_casting_type: u32, // 0 = usual rendering, 1 = sphere ray casting, 2 = cylinder ray casting
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) normal: vec3<f32>,
    @location(1) color: vec4<f32>,
    @location(2) ray_casting_type: u32,
    @location(3) ray_casting_scale: vec3<f32>,
    @location(4) sphere_center_view: vec3<f32>,
    @location(5) vertex_pos_view: vec3<f32>,
};

struct FragmentOutput {
    @location(0) color: vec4<f32>,
    @builtin(frag_depth) depth: f32,
}

fn get_scale(matrix: mat4x4<f32>) -> vec3<f32> {
    return vec3<f32>(
        length(vec3<f32>(matrix[0][0], matrix[0][1], matrix[0][2])),
        length(vec3<f32>(matrix[1][0], matrix[1][1], matrix[1][2])),
        length(vec3<f32>(matrix[2][0], matrix[2][1], matrix[2][2]))
    );
}

fn ray_casting_sphere_position(vertex: VertexInput, instance: InstanceInput) -> VertexOutput {
    let model_transform = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );

    var output: VertexOutput;
    output.position = uniforms.final_transform * model_transform * vec4<f32>(vertex.position, 1.0);
    output.color = instance.color;

    // Extract scale components from model and scene matrices
    let model_scale = get_scale(model_transform);
    let scene_scale = get_scale(uniforms.scene_transform);

    // For sphere: use X scale as radius
    // For cylinder: use X/Y scale as radius, Z scale as half-height
    output.ray_casting_scale = vec3<f32>(
        model_scale.x * scene_scale.x,
        model_scale.y * scene_scale.y,
        model_scale.z * scene_scale.z
    );

    // Transform sphere center to view space
    let sphere_center_world = (uniforms.scene_transform * model_transform * vec4<f32>(0.0, 0.0, 0.0, 1.0)).xyz;
    output.sphere_center_view = (uniforms.view_transform * vec4<f32>(sphere_center_world, 1.0)).xyz;

    // Transform vertex position to view space
    let vertex_world = (uniforms.scene_transform * model_transform * vec4<f32>(vertex.position, 1.0)).xyz;
    output.vertex_pos_view = (uniforms.view_transform * vec4<f32>(vertex_world, 1.0)).xyz;

    // Calculate normal matrix for transforming normals from object space to view space
    // let model_view_matrix = uniforms.view_transform * uniforms.scene_transform * model_transform;
    // let normal_matrix_view = transpose(inverse(model_view_matrix));

    return output;
}

fn default_position(vertex: VertexInput, instance: InstanceInput) -> VertexOutput {
    let model_transform = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );

    var output: VertexOutput;
    output.position = uniforms.final_transform * model_transform * vec4<f32>(vertex.position, 1.0);
    output.normal = (uniforms.scene_transform * model_transform * vec4<f32>(vertex.normal, 0.0)).xyz;
    output.color = instance.color;
    output.ray_casting_scale = vec3<f32>(0.0, 0.0, 0.0);
    return output;
}

@vertex
fn vs_main(vertex: VertexInput, instance: InstanceInput) -> VertexOutput {
    var output: VertexOutput;
    if (instance.ray_casting_type == 1u) {
        output = ray_casting_sphere_position(vertex, instance);
    }
    else {
        output = default_position(vertex, instance);
    }
    output.ray_casting_type = instance.ray_casting_type;
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

fn ray_casting_sphere(in: VertexOutput) -> vec3<f32> {
    var ray_origin: vec3<f32>;
    var ray_dir: vec3<f32>;

    if (uniforms.is_perspective == 1) {
        // For perspective projection: rays from camera origin
        ray_origin = vec3<f32>(0.0, 0.0, 0.0);
        ray_dir = normalize(in.vertex_pos_view);
    } else {
        // For orthographic projection: parallel rays
        // Ray origin is on the fragment's XY position in view space, far from camera
        ray_origin = vec3<f32>(in.vertex_pos_view.xy, 0.0);
        ray_dir = vec3<f32>(0.0, 0.0, -1.0);        
    }

    // Sphere equation: |P - C|^2 = r^2
    // Ray equation: P = O + t*D
    // Solving: |O + t*D - C|^2 = r^2
    // Let OC = O - C (vector from sphere center to ray origin)
    let oc = ray_origin - in.sphere_center_view;

    // Quadratic equation: at^2 + bt + c = 0
    let a: f32 = dot(ray_dir, ray_dir);
    let b: f32 = 2.0 * dot(oc, ray_dir);
    let c: f32 = dot(oc, oc) - in.ray_casting_scale.x * in.ray_casting_scale.x;

    let discriminant: f32 = b * b - 4.0 * a * c;

    // No intersection - discard fragment
    if (discriminant < 0.0) {
        discard;
    }

    // Find nearest intersection
    let t: f32 = (-b - sqrt(discriminant)) / (2.0 * a);

    // If intersection is behind the camera, discard
    if (t < 0.0) {
        discard;
    }

    // Calculate intersection point in view space
    return ray_origin + ray_dir * t;
}

@fragment
fn fs_main(in: VertexOutput) -> FragmentOutput {
    var output: FragmentOutput;
    var normal: vec3<f32>;

    if (in.ray_casting_type == 1u) {
        let intersection_point = ray_casting_sphere(in);
        normal = normalize(intersection_point - in.sphere_center_view);

        let clip_space_pos: vec4<f32> = uniforms.projection_transform * vec4<f32>(intersection_point, 1.0);
        output.depth = clip_space_pos.z / clip_space_pos.w;
    } else {
        normal = in.normal;
        output.depth = in.position.z;
    }

    let ambient_strength: f32 = 0.3;
    let specular_strength: f32 = 0.6;
    let shininess: f32 = 16.0;
    let light_color = vec3<f32>(1.0, 1.0, 1.0) * 0.9;
    let light_position = vec3<f32>(0.3, 0.3, 1.0);

    output.color = calculate_blinn_phong(
        in.color, 
        normal,
        light_color, 
        light_position, 
        ambient_strength, 
        specular_strength, 
        shininess
    );

    return output;
}
