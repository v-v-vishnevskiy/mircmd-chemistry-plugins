struct Uniforms {
    view_projection: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec3<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    // Triangle vertices in world space
    var positions = array<vec3<f32>, 3>(
        vec3<f32>(0.0, 0.5, 0.0),    // Top
        vec3<f32>(-0.5, -0.5, 0.0),  // Bottom left
        vec3<f32>(0.5, -0.5, 0.0),   // Bottom right
    );

    // Vertex colors (RGB)
    var colors = array<vec3<f32>, 3>(
        vec3<f32>(1.0, 0.0, 0.0),  // Red
        vec3<f32>(0.0, 1.0, 0.0),  // Green
        vec3<f32>(0.0, 0.0, 1.0),  // Blue
    );

    var output: VertexOutput;
    output.position = uniforms.view_projection * vec4<f32>(positions[vertex_index], 1.0);
    output.color = colors[vertex_index];
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(input.color, 1.0);
}
