pub mod shader_config;

pub mod graphics_backend_client;
pub mod graphics_backend_worker;

pub mod legacy_graphics;

pub const DEFAULT_COMPUTE: &str = include_str!("compute.wgsl");
