struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@group(0)
@binding(0)
var tex: texture_2d<f32>;

@group(0)
@binding(1)
var tex_sampler: sampler;

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    
    // Create a fullscreen triangle
    // This maps 0 -> (0, 0), 1 -> (2, 0), 2 -> (0, 2)
    let x = f32((in_vertex_index << 1u) & 2u);
    let y = f32(in_vertex_index & 2u);

    // This maps (0,0) -> (-1, -1, ...), (2, 0) -> (3, -1, ...), (0, 2) -> (-1, 3, ...) 
    out.clip_position = vec4<f32>(x * 2.0 - 1.0, y * 2.0 - 1.0, 0.0, 1.0);
    // This maps (0, 0) -> (0, 1), (2, 0) -> (2, 1), (0, 2) -> (0, -1)
    out.uv = vec2<f32>(x, 1.0 - y);

    return out;
}

@fragment
fn fs_color(in: VertexOutput) -> @location(0) vec4<f32> {
    let sample = textureSample(tex, tex_sampler, in.uv);
    return sample;
}