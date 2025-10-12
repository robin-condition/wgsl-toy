// https://sotrh.github.io/learn-wgpu/beginner/tutorial3-pipeline/#writing-the-shaders

@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32
) -> @builtin(position) vec4<f32> {
    if in_vertex_index == 0 {
        return vec4<f32>(-1.0, -1.0, 0.0, 1.0);
    }
    if in_vertex_index == 1 {
        return vec4<f32>(3.0, -1.0, 0.0, 1.0);
    }
    return vec4<f32>(-1.0, 3.0, 0.0, 1.0);
}
