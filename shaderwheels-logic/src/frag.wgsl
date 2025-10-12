// Fragment shader
// https://sotrh.github.io/learn-wgpu/beginner/tutorial3-pipeline/#writing-the-shaders
@group(0) @binding(0) var<uniform> size: vec4<u32>;

@fragment
fn main(@builtin(position) in: vec4<f32>) -> @location(0) vec4<f32> {
    return vec4<f32>(in.x / f32(size.x), in.y / f32(size.y), 0.0, 1.0);
}
