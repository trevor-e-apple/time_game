// Vertex shader

struct CameraUniform {
    view_proj: mat4x4<f32>,
};
@group(1) @binding(0) var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) tex_coords: vec2<f32>,
};

struct InstanceInput {
    @location(2) model_matrix_0: vec3<f32>,
    @location(3) model_matrix_1: vec3<f32>,
    @location(4) model_matrix_2: vec3<f32>,
}


struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

@vertex
fn vs_main(
    model: VertexInput, instance: InstanceInput,
) -> VertexOutput {
    let model_matrix = mat3x3<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
    );

    var out: VertexOutput;
    out.tex_coords = model.tex_coords;

    let position = model_matrix * vec3<f32>(model.position.x, model.position.y, 1.0);
    out.clip_position = camera.view_proj * vec4<f32>(position.x, position.y, 0.0, 1.0);
    return out;
}

// Fragment shader
@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(t_diffuse, s_diffuse, in.tex_coords);
}
