use std::sync::mpsc::Receiver;

use cardigan_incremental::{ReceivedVersioned, Versioned, memoized};
use wgpu::TextureView;

use crate::rendering::{
    graphics_backend_worker::{
        compute_worker::ComputeWorkerPart,
        shared::{BackendWorker, GPUAdapterInfo},
    },
    shader_config::{ShaderConfig, ShaderLanguage},
};

mod compute_worker;
mod fragment_worker;
mod shared;

enum ArbitraryWorker {
    ComputeWorker(ComputeWorkerPart),
    FragmentWorker(),
}

impl BackendWorker for ArbitraryWorker {
    async fn step(
        &mut self,
        preout_size: &Versioned<(u32, u32)>,
        hardware: &Versioned<&GPUAdapterInfo>,
        module: &Versioned<&wgpu::ShaderModule>,
        entry_point: &Versioned<&String>,
        blitter: &Versioned<&wgpu::util::TextureBlitter>,
        render_output_on_invalidated: bool,
        output_view: &Versioned<&TextureView>,
    ) -> bool {
        match self {
            ArbitraryWorker::ComputeWorker(compute_worker_part) => {
                compute_worker_part
                    .step(
                        preout_size,
                        hardware,
                        module,
                        entry_point,
                        blitter,
                        render_output_on_invalidated,
                        output_view,
                    )
                    .await
            }
            ArbitraryWorker::FragmentWorker() => todo!(),
        }
    }
}

pub struct SettingsReceivers {
    pub shader_content_recvr: ReceivedVersioned<ShaderConfig>,

    pub hardware_recvr: ReceivedVersioned<GPUAdapterInfo>,
    pub output_texture_view_recvr: ReceivedVersioned<TextureView>,
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
    backend: Versioned<ArbitraryWorker>,
}

impl Worker {
    fn read_recvrs(&mut self) {}

    async fn step(&mut self) {}
}
