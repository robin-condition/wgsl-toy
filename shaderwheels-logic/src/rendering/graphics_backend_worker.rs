use std::sync::mpsc::Receiver;

use wgpu::TextureView;

use crate::rendering::{
    graphics_backend_worker::compute_worker::GPUAdapterInfo, shader_config::{ShaderConfig, ShaderLanguage},
};

mod compute_worker;
mod fragment_worker;

trait BackendWorker {
    fn invalidate_shader_contents(&mut self);
    fn invalidate_shaderlanguage(&mut self);
    fn invalidate_append_environment(&mut self);
    fn invalidate_hardware(&mut self);
}

enum ArbitraryWorker {
    ComputeWorker(),
    FragmentWorker(),
}

pub struct SettingsReceivers {
    pub shader_text_recvr: Receiver<Option<String>>,
    pub shader_lang_recvr: Receiver<Option<ShaderLanguage>>,
    pub append_env_recvr: Receiver<bool>,

    pub complete_shader_ctx_receiver: Receiver<Option<ShaderConfig>>,

    pub hardware_recvr: Receiver<GPUAdapterInfo>,
    pub output_texture_view_recvr: Receiver<TextureView>,
}

pub struct Settings {
    pub shader_text: Option<String>,
    pub append_environment: bool,
    pub shader_lang: Option<ShaderLanguage>,
    pub hardware: GPUAdapterInfo,
    pub output_texture_view: TextureView,
}

pub struct Worker {
    settings_recvrs: SettingsReceivers,
    settings: Settings,
    backend: ArbitraryWorker,
}

impl Worker {

    fn read_recvrs(&mut self) {
        self.settings_recvrs.append_env_recvr.
    }


    async fn step(&mut self) {

    }
}
