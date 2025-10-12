use wgpu::{Device, Queue};

#[derive(Clone, Copy)]
pub enum ShaderLanguage {
    Wgsl,
    //Glsl,
}

#[derive(Clone)]
pub enum ShaderBackend {
    FullCompute,
    FullFragment,
    General,
}

#[derive(Clone)]
pub struct ShaderConfig {
    pub content: String,
    pub language: ShaderLanguage,
    pub backend: ShaderBackend,
}

impl Default for ShaderConfig {
    fn default() -> Self {
        Self {
            //content: crate::rendering::DEFAULT_WGSL_COMPUTE.to_string(),
            content: crate::rendering::DEFAULT_WGSL_FRAG.to_string(),
            language: ShaderLanguage::Wgsl,
            // Should be switched to general once support exists
            //backend: ShaderBackend::FullCompute,
            backend: ShaderBackend::FullFragment,
        }
    }
}

pub struct GPUAdapterInfo {
    pub deviceref: Device,
    pub queueref: Queue,
}
