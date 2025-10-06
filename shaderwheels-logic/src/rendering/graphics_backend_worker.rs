use std::sync::mpsc::Receiver;

use crate::rendering::{
    graphics_backend_worker::compute_worker::GPUAdapterInfo, shader_config::ShaderLanguage,
};

mod compute_worker;
mod fragment_worker;

enum ArbitraryWorker {
    ComputeWorker(),
    FragmentWorker(),
}

pub struct SettingsReceivers {
    pub shader_text_recvr: Receiver<String>,
    pub shader_lang_recvr: Receiver<ShaderLanguage>,

    pub hardware_recvr: Receiver<GPUAdapterInfo>,
}

pub struct Settings {
    pub shader_text: String,
    pub shader_lang: ShaderLanguage,
    pub hardware: GPUAdapterInfo,
}

pub struct Worker {
    settings_recvrs: SettingsReceivers,
    backend: ArbitraryWorker,
}
