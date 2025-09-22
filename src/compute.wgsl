
@group(0) @binding(0)
//var<storage, write> output: texture_2d<
var textureOutput: texture_storage_2d<rgba8unorm, write>;

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {

    let dimensions = textureDimensions(textureOutput);

    if (global_id.x >= dimensions.x || global_id.y >= dimensions.y) {
        return;
    }
    
    textureStore(textureOutput, vec2<i32>(i32(global_id.x), i32(global_id.y)), vec4<f32>(0.0, 0.5, 0.0, 1.0));
}