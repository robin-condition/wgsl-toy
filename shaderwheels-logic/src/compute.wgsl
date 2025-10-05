
@group(0) @binding(0)
//var<storage, write> output: texture_2d<
var textureOutput: texture_storage_2d<rgba8unorm, write>;

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {

    let dimensions = textureDimensions(textureOutput);

    if (global_id.x >= dimensions.x || global_id.y >= dimensions.y) {
        return;
    }

    let uv = vec2<f32>(f32(global_id.x), f32(global_id.y)) / vec2<f32>(f32(dimensions.x), f32(dimensions.y));

    let color: vec3<f32> = vec3<f32>(uv.x, uv.y, 1.0 - uv.x);
    
    textureStore(textureOutput, vec2<i32>(i32(global_id.x), i32(global_id.y)), vec4<f32>(color, 1.0));
}