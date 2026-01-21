// Vertex shader

struct VertexInput {
    @location(0) position: vec2<f32>,
};

struct InstanceInput {
    @location(2) model_matrix_0: vec3<f32>,
    @location(3) model_matrix_1: vec3<f32>,
    @location(4) model_matrix_2: vec3<f32>,
    @location(5) color: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
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
    out.color = instance.color;

    let position = model_matrix * vec3<f32>(model.position.x, model.position.y, 1.0);

    out.clip_position = vec4<f32>(position.x, position.y, 0.0, 1.0);
    return out;
}

// Fragment shader

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}
