pub enum ShaderLanguage {
    Wgsl,
    Glsl,
}

pub enum ShaderBackend {
    FullCompute,
    FullFragment,
    General,
}

pub struct ShaderConfig {
    pub content: String,
    pub language: ShaderLanguage,
    pub backend: ShaderBackend,
}
