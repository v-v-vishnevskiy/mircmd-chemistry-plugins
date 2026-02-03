struct Uniforms {
    projection_transform: mat4x4<f32>,
    view_transform: mat4x4<f32>,
    scene_transform: mat4x4<f32>,
    final_transform: mat4x4<f32>, // projection_transform * view_transform * scene_transform
    render_mode: u32,             // 0 = normal, 1 = picking
    is_perspective: u32,          // 0 = orthographic, 1 = perspective
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@group(0) @binding(1)
var font_atlas: texture_2d<f32>;

@group(0) @binding(2)
var font_sampler: sampler;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) tex_coord: vec2<f32>,
};

struct InstanceInput {
    @location(3) model_matrix_0: vec4<f32>,
    @location(4) model_matrix_1: vec4<f32>,
    @location(5) model_matrix_2: vec4<f32>,
    @location(6) model_matrix_3: vec4<f32>,
    @location(7) color: vec4<f32>,
    @location(8) picking_color: vec4<f32>,
    @location(9) lighting_model: u32,    // 0 = flat color, 1 = Blinn Phong
    @location(10) ray_casting_type: u32, // 0 = usual rendering, 1 = sphere ray casting, 2 = cylinder ray casting
    @location(11) use_texture: u32,      // 0 = no texture, 1 = sample from font atlas
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) normal: vec3<f32>,
    @location(1) color: vec4<f32>,
    @location(2) @interpolate(flat) lighting_model: u32,
    @location(3) @interpolate(flat) ray_casting_type: u32,
    @location(4) ray_casting_scale: vec3<f32>,
    @location(5) sphere_center_view: vec3<f32>,
    @location(6) vertex_pos_view: vec3<f32>,
    @location(7) cylinder_axis_view: vec3<f32>,
    @location(8) tex_coord: vec2<f32>,
    @location(9) @interpolate(flat) use_texture: u32,
};

struct RayCastingOutput {
    intersection_point: vec3<f32>,
    normal: vec3<f32>,
}

struct FragmentOutput {
    @location(0) color: vec4<f32>,
    @builtin(frag_depth) depth: f32,
}

// WBOIT (Weighted Blended Order-Independent Transparency) output
struct WboitFragmentOutput {
    @location(0) accumulation: vec4<f32>,  // RGB * A * weight, A * weight
    @location(1) revealage: f32,           // Product of (1 - alpha)
}

// WBOIT weight function based on McGuire and Bavoil paper
fn wboit_weight(color: vec4<f32>, depth: f32) -> f32 {
    // Attempt to prevent floating-point overflow and underflow
    let a = min(1.0, color.a) * 8.0 + 0.01;
    let b = -depth * 0.95 + 1.0;

    // Weight based on alpha and depth
    // Objects closer to camera (smaller depth) get higher weight
    let weight = clamp(a * a * a * 1e8 * b * b * b, 1e-2, 3e2);
    return weight;
}

fn get_scale(matrix: mat4x4<f32>) -> vec3<f32> {
    return vec3<f32>(
        length(vec3<f32>(matrix[0][0], matrix[0][1], matrix[0][2])),
        length(vec3<f32>(matrix[1][0], matrix[1][1], matrix[1][2])),
        length(vec3<f32>(matrix[2][0], matrix[2][1], matrix[2][2]))
    );
}

fn inverse(m: mat4x4<f32>) -> mat4x4<f32> {
    let a00 = m[0][0]; let a01 = m[0][1]; let a02 = m[0][2]; let a03 = m[0][3];
    let a10 = m[1][0]; let a11 = m[1][1]; let a12 = m[1][2]; let a13 = m[1][3];
    let a20 = m[2][0]; let a21 = m[2][1]; let a22 = m[2][2]; let a23 = m[2][3];
    let a30 = m[3][0]; let a31 = m[3][1]; let a32 = m[3][2]; let a33 = m[3][3];

    let b00 = a00 * a11 - a01 * a10;
    let b01 = a00 * a12 - a02 * a10;
    let b02 = a00 * a13 - a03 * a10;
    let b03 = a01 * a12 - a02 * a11;
    let b04 = a01 * a13 - a03 * a11;
    let b05 = a02 * a13 - a03 * a12;
    let b06 = a20 * a31 - a21 * a30;
    let b07 = a20 * a32 - a22 * a30;
    let b08 = a20 * a33 - a23 * a30;
    let b09 = a21 * a32 - a22 * a31;
    let b10 = a21 * a33 - a23 * a31;
    let b11 = a22 * a33 - a23 * a32;

    let det = b00 * b11 - b01 * b10 + b02 * b09 + b03 * b08 - b04 * b07 + b05 * b06;
    let inv_det = 1.0 / det;

    return mat4x4<f32>(
        vec4<f32>(
            (a11 * b11 - a12 * b10 + a13 * b09) * inv_det,
            (a02 * b10 - a01 * b11 - a03 * b09) * inv_det,
            (a31 * b05 - a32 * b04 + a33 * b03) * inv_det,
            (a22 * b04 - a21 * b05 - a23 * b03) * inv_det
        ),
        vec4<f32>(
            (a12 * b08 - a10 * b11 - a13 * b07) * inv_det,
            (a00 * b11 - a02 * b08 + a03 * b07) * inv_det,
            (a32 * b02 - a30 * b05 - a33 * b01) * inv_det,
            (a20 * b05 - a22 * b02 + a23 * b01) * inv_det
        ),
        vec4<f32>(
            (a10 * b10 - a11 * b08 + a13 * b06) * inv_det,
            (a01 * b08 - a00 * b10 - a03 * b06) * inv_det,
            (a30 * b04 - a31 * b02 + a33 * b00) * inv_det,
            (a21 * b02 - a20 * b04 - a23 * b00) * inv_det
        ),
        vec4<f32>(
            (a11 * b07 - a10 * b09 - a12 * b06) * inv_det,
            (a00 * b09 - a01 * b07 + a02 * b06) * inv_det,
            (a31 * b01 - a30 * b03 - a32 * b00) * inv_det,
            (a20 * b03 - a21 * b01 + a22 * b00) * inv_det
        )
    );
}

fn ray_casting_position(vertex: VertexInput, instance: InstanceInput) -> VertexOutput {
    let model_transform = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );

    var output: VertexOutput;
    output.position = uniforms.final_transform * model_transform * vec4<f32>(vertex.position, 1.0);
    output.tex_coord = vertex.tex_coord;

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

    // Calculate cylinder axis in view space (Z-axis transformed)
    let model_view_matrix = uniforms.view_transform * uniforms.scene_transform * model_transform;
    let normal_matrix = transpose(inverse(model_view_matrix));
    output.cylinder_axis_view = normalize((normal_matrix * vec4<f32>(0.0, 0.0, 1.0, 0.0)).xyz);

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
    output.tex_coord = vertex.tex_coord;
    output.ray_casting_scale = vec3<f32>(0.0, 0.0, 0.0);
    output.sphere_center_view = vec3<f32>(0.0, 0.0, 0.0);
    output.vertex_pos_view = vec3<f32>(0.0, 0.0, 0.0);
    output.cylinder_axis_view = vec3<f32>(0.0, 0.0, 1.0);
    return output;
}

@vertex
fn vs_main(vertex: VertexInput, instance: InstanceInput) -> VertexOutput {
    var output: VertexOutput;
    switch instance.ray_casting_type {
        case 1u, 2u {
            output = ray_casting_position(vertex, instance);
        }
        default {
            output = default_position(vertex, instance);
        }
    }

    switch uniforms.render_mode {
        case 1u {
            output.color = instance.picking_color;
        }
        default {
            output.color = instance.color;
        }
    }

    output.lighting_model = instance.lighting_model;
    output.ray_casting_type = instance.ray_casting_type;
    output.use_texture = instance.use_texture;
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

fn ray_casting_sphere(in: VertexOutput, ray_origin: vec3<f32>, ray_dir: vec3<f32>) -> RayCastingOutput {
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

    var result: RayCastingOutput;
    result.intersection_point = ray_origin + ray_dir * t;
    result.normal = normalize(result.intersection_point - in.sphere_center_view);

    return result;
}

fn ray_casting_cylinder(in: VertexOutput, ray_origin: vec3<f32>, ray_dir: vec3<f32>) -> RayCastingOutput {
    // Cylinder parameters in view space
    let cyl_radius = in.ray_casting_scale.x;
    let cyl_half_height = in.ray_casting_scale.z;
    let cyl_center = in.sphere_center_view;

    // Vector from ray origin to cylinder center
    let oc = ray_origin - cyl_center;

    // Get cylinder axis in view space (precomputed in vertex shader)
    let cyl_axis: vec3f = normalize(in.cylinder_axis_view);

    // Project vectors onto plane perpendicular to cylinder axis
    let oc_perp = oc - dot(oc, cyl_axis) * cyl_axis;
    let rd_perp = ray_dir - dot(ray_dir, cyl_axis) * cyl_axis;

    var t_final: f32 = -1.0;
    var final_normal = vec3f(0.0, 0.0, 0.0);

    // --- 1. Check cylinder side ---
    let a: f32 = dot(rd_perp, rd_perp);
    let b: f32 = 2.0 * dot(oc_perp, rd_perp);
    let c: f32 = dot(oc_perp, oc_perp) - cyl_radius * cyl_radius;

    let delta = b * b - 4.0 * a * c;

    if (delta >= 0.0 && abs(a) > 0.0001) {
        let t = (-b - sqrt(delta)) / (2.0 * a);
        if (t > 0.0) {
            let hit_point = ray_origin + t * ray_dir;
            let height = dot(hit_point - cyl_center, cyl_axis);

            // Check if within cylinder height
            if (abs(height) <= cyl_half_height) {
                t_final = t;
                // Normal is perpendicular to axis, pointing outward
                let point_on_axis = cyl_center + height * cyl_axis;
                final_normal = normalize(hit_point - point_on_axis);
            }
        }
    }

    // --- 2. Check caps ---
    let denom = dot(ray_dir, cyl_axis);
    if (abs(denom) > 0.0001) {
        // Top cap
        let t_top = dot(cyl_center + cyl_half_height * cyl_axis - ray_origin, cyl_axis) / denom;
        if (t_top > 0.0) {
            let p = ray_origin + t_top * ray_dir;
            let v = p - (cyl_center + cyl_half_height * cyl_axis);
            if (dot(v, v) <= cyl_radius * cyl_radius) {
                if (t_final < 0.0 || t_top < t_final) {
                    t_final = t_top;
                    final_normal = cyl_axis;
                }
            }
        }

        // Bottom cap
        let t_bot = dot(cyl_center - cyl_half_height * cyl_axis - ray_origin, cyl_axis) / denom;
        if (t_bot > 0.0) {
            let p = ray_origin + t_bot * ray_dir;
            let v = p - (cyl_center - cyl_half_height * cyl_axis);
            if (dot(v, v) <= cyl_radius * cyl_radius) {
                if (t_final < 0.0 || t_bot < t_final) {
                    t_final = t_bot;
                    final_normal = -cyl_axis;
                }
            }
        }
    }

    // --- 3. Result ---
    if (t_final < 0.0) {
        discard;
    }

    var result: RayCastingOutput;
    result.intersection_point = ray_origin + ray_dir * t_final;
    result.normal = final_normal;

    return result;
}

fn ray_casting(in: VertexOutput) -> RayCastingOutput {
    var ray_origin: vec3<f32>;
    var ray_dir: vec3<f32>;

    if (uniforms.is_perspective == 1u) {
        // For perspective projection: rays from camera origin
        ray_origin = vec3f(0.0, 0.0, 0.0);
        ray_dir = normalize(in.vertex_pos_view);
    } else {
        // For orthographic projection: parallel rays
        ray_origin = vec3f(in.vertex_pos_view.xy, 0.0);
        ray_dir = vec3(0.0, 0.0, -1.0);
    }

    var result: RayCastingOutput;
    switch in.ray_casting_type {
        case 1u {
            result = ray_casting_sphere(in, ray_origin, ray_dir);
        }
        case 2u {
            result = ray_casting_cylinder(in, ray_origin, ray_dir);
        }
        default {
            discard;
        }
    }
    return result;
}

fn calculate_fragment_color(in: VertexOutput, normal: vec3<f32>) -> vec4<f32> {
    var color: vec4<f32>;
    switch uniforms.render_mode {
        case 1u { // picking mode
            color = in.color;
        }
        default {
            switch in.lighting_model {
                case 1u {
                    let ambient_strength: f32 = 0.3;
                    let specular_strength: f32 = 0.6;
                    let shininess: f32 = 16.0;
                    let light_color = vec3<f32>(1.0, 1.0, 1.0) * 0.9;
                    let light_position = vec3<f32>(0.3, 0.3, 1.0);

                    color = calculate_blinn_phong(
                        in.color, 
                        normal,
                        light_color, 
                        light_position, 
                        ambient_strength, 
                        specular_strength, 
                        shininess
                    );
                }
                default {
                    color = in.color;
                }
            }
        }
    }
    
    // Apply font atlas texture if use_texture is set
    if (in.use_texture == 1u) {
        let uv = vec2<f32>(in.tex_coord.x, 1.0 - in.tex_coord.y);
        let texture_alpha = textureSample(font_atlas, font_sampler, uv).r;
        if (texture_alpha < 0.01) {
            discard;
        }
        color = vec4<f32>(color.rgb, color.a * texture_alpha);
    }

    return color;
}

@fragment
fn fs_main(in: VertexOutput) -> FragmentOutput {
    var output: FragmentOutput;
    var normal: vec3<f32>;

    switch in.ray_casting_type {
        case 1u, 2u {
            let rc_out = ray_casting(in);
            normal = rc_out.normal;

            let clip_space_pos: vec4<f32> = uniforms.projection_transform * vec4<f32>(rc_out.intersection_point, 1.0);
            output.depth = clip_space_pos.z / clip_space_pos.w;
        }
        default {
            normal = in.normal;
            output.depth = in.position.z;
        }
    }

    output.color = calculate_fragment_color(in, normal);
    return output;
}

// Fragment shader for transparent objects (WBOIT)
@fragment
fn fs_transparent(in: VertexOutput) -> WboitFragmentOutput {
    var output: WboitFragmentOutput;
    var normal: vec3<f32>;
    var depth: f32;

    switch in.ray_casting_type {
        case 1u, 2u {
            let rc_out = ray_casting(in);
            normal = rc_out.normal;

            let clip_space_pos: vec4<f32> = uniforms.projection_transform * vec4<f32>(rc_out.intersection_point, 1.0);
            depth = clip_space_pos.z / clip_space_pos.w;
        }
        default {
            normal = in.normal;
            depth = in.position.z;
        }
    }

    let color = calculate_fragment_color(in, normal);

    // Calculate WBOIT weight
    let weight = wboit_weight(color, depth);

    // Output to accumulation buffer: premultiplied color * weight, alpha * weight
    output.accumulation = vec4<f32>(color.rgb * color.a, color.a) * weight;

    // Output to revealage buffer: (1 - alpha) for multiplicative blending
    output.revealage = color.a;

    return output;
}
