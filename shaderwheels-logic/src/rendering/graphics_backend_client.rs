use crate::rendering::shader_config::ShaderConfig;

pub struct GraphicsClient {
    // Inputs
    preout_size: (u32, u32),
    shader_info: ShaderConfig,
}
