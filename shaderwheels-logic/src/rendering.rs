pub mod shader_config;

pub mod communication;
pub mod graphics_backend_client;
pub mod graphics_backend_worker;

pub const DEFAULT_WGSL_COMPUTE: &str = include_str!("compute.wgsl");
pub const WGSL_ENTRY: &str = "main";

pub const DEFAULT_WGSL_VERT: &str = include_str!("vert.wgsl");
pub const WGSL_VERT_ENTRY: &str = "vs_main";

pub const DEFAULT_WGSL_FRAG: &str = include_str!("frag.wgsl");
pub const WGSL_FRAG_ENTRY: &str = "fs_main";
