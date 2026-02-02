// Weighted Blended Order-Independent Transparency (WBOIT) Composite Shader
// Based on: McGuire and Bavoil, "Weighted Blended Order-Independent Transparency"

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
};

@group(0) @binding(0)
var accumulation_texture: texture_2d<f32>;

@group(0) @binding(1)
var revealage_texture: texture_2d<f32>;

// Full-screen quad vertices (two triangles)
var<private> positions: array<vec2<f32>, 6> = array<vec2<f32>, 6>(
    vec2<f32>(-1.0, -1.0),
    vec2<f32>(1.0, -1.0),
    vec2<f32>(1.0, 1.0),
    vec2<f32>(-1.0, -1.0),
    vec2<f32>(1.0, 1.0),
    vec2<f32>(-1.0, 1.0),
);

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var output: VertexOutput;
    output.position = vec4<f32>(positions[vertex_index], 0.0, 1.0);
    return output;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let coords = vec2<i32>(in.position.xy);

    // Sample accumulation and revealage textures
    let accum = textureLoad(accumulation_texture, coords, 0);
    let revealage = textureLoad(revealage_texture, coords, 0).r;

    // If revealage is 1.0, there are no transparent fragments
    if (revealage >= 1.0) {
        discard;
    }

    // Prevent division by zero
    let accum_alpha = max(accum.a, 0.00001);

    // Calculate average color
    let average_color = accum.rgb / accum_alpha;

    // Final color with alpha from revealage
    let alpha = 1.0 - revealage;

    return vec4<f32>(average_color, alpha);
}
