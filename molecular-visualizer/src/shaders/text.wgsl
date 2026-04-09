struct Uniforms {
    projection_transform: mat4x4<f32>,
    view_transform: mat4x4<f32>,
    scene_transform: mat4x4<f32>,
    final_transform: mat4x4<f32>,
    render_mode: u32,
    is_perspective: u32,
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
    @location(7) uv_rect: vec4<f32>,
    @location(8) width: f32,
    @location(9) char_x_offset: f32,
    @location(10) depth_offset: f32,
    @location(11) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) tex_coord: vec2<f32>,
};

struct FragmentOutput {
    @location(0) color: vec4<f32>,
}

struct WboitFragmentOutput {
    @location(0) accumulation: vec4<f32>,
    @location(1) revealage: f32,
}

fn wboit_weight(color: vec4<f32>, depth: f32) -> f32 {
    let a = min(1.0, color.a) * 8.0 + 0.01;
    let b = -depth * 0.95 + 1.0;
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

@vertex
fn vs_main(vertex: VertexInput, instance: InstanceInput) -> VertexOutput {
    let model_transform = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );

    let view_scale = get_scale(uniforms.view_transform);
    let scene_scale = get_scale(uniforms.scene_transform);
    let model_scale = get_scale(model_transform);

    let scale = model_scale * scene_scale * view_scale;

    // Extract billboard center position from instance model matrix (world position basically before view)
    let billboard_center = vec3<f32>(model_transform[3][0], model_transform[3][1], model_transform[3][2]);

    // Transform center to view space
    let view_center = uniforms.view_transform * uniforms.scene_transform * vec4<f32>(billboard_center, 1.0);

    // Apply billboard transformation: offset vertex in view space
    var view_position = view_center;
    // position is offsetted by char local position (x-axis)
    view_position.x += ((vertex.position.x * instance.width) + instance.char_x_offset) * scale.x;
    view_position.y += vertex.position.y * scale.y;

    var output: VertexOutput;
    var clip_position = uniforms.projection_transform * view_position;

    // Apply depth hack: Calculate a new clip position that is closer to the camera
    var offset_view_position = view_position;
    // We assume the camera looks down -Z in view space, so adding to Z moves it closer.
    // instance.depth_offset is in molecule's model space, so we must scale it by scene_scale 
    // to match the sphere's actual scaled radius in view space!
    offset_view_position.z += instance.depth_offset * scene_scale.z * view_scale.z;
    let offset_clip_position = uniforms.projection_transform * offset_view_position;
    
    // Calculate the target depth [0, 1] for the closer position
    let target_depth = offset_clip_position.z / offset_clip_position.w;

    // Apply the target depth to our original clip position
    // This changes the Z value written to the depth buffer, but keeps X, Y, W the same,
    // so the billboard's visual size on screen doesn't change!
    clip_position.z = target_depth * clip_position.w;

    output.position = clip_position;

    output.color = instance.color;

    let u = mix(instance.uv_rect[0], instance.uv_rect[2], vertex.tex_coord.x);
    let v = mix(instance.uv_rect[1], instance.uv_rect[3], vertex.tex_coord.y);
    output.tex_coord = vec2<f32>(u, v);

    return output;
}

fn calculate_fragment_color(in: VertexOutput) -> vec4<f32> {
    var color = in.color;
    let uv = vec2<f32>(in.tex_coord.x, 1.0 - in.tex_coord.y);
    let texture_alpha = textureSample(font_atlas, font_sampler, uv).r;
    if (texture_alpha < 0.01) {
        discard;
    }
    return vec4<f32>(color.rgb, color.a * texture_alpha);
}

@fragment
fn fs_main(in: VertexOutput) -> FragmentOutput {
    var output: FragmentOutput;
    output.color = calculate_fragment_color(in);
    return output;
}

@fragment
fn fs_transparent(in: VertexOutput) -> WboitFragmentOutput {
    var output: WboitFragmentOutput;
    let color = calculate_fragment_color(in);
    let depth = in.position.z;

    let weight = wboit_weight(color, depth);

    output.accumulation = vec4<f32>(color.rgb * color.a, color.a) * weight;
    output.revealage = color.a;

    return output;
}
