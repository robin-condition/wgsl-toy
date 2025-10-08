use std::sync::mpsc::Receiver;

use cardigan_incremental::{memoized, ReceivedVersioned, Versioned, VersionedInputs};
use wgpu::{TextureFormat, TextureView};

use crate::rendering::{
    graphics_backend_worker::{
        compute_worker::ComputeWorkerPart,
        shared::{blitter, module_comp, BackendWorker, GPUAdapterInfo},
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
    pub preout_size: (u32, u32),
    pub hardware: GPUAdapterInfo,
    pub output_texture_view: TextureView,
}

pub struct VersionedSettings {
    pub shader_text: Versioned<String>,
    pub append_environment: Versioned<bool>,
    pub shader_lang: Versioned<ShaderLanguage>,
    pub preout_size: Versioned<(u32, u32)>,
    pub hardware: Versioned<GPUAdapterInfo>,
    pub output_texture_view: Versioned<TextureView>,
    pub output_texture_format: Versioned<TextureFormat>
}

pub struct Worker {
    settings_recvrs: SettingsReceivers,
    settings: VersionedSettings,
    backend: ArbitraryWorker,

    render_on_invalid: bool,

    blitter: blitter,
    mod_comp: module_comp,
    when_send_comp_errs: VersionedInputs<1>
}

impl Worker {
    fn read_recvrs(&mut self) {}

    async fn step(&mut self) {

        let hardware = self.settings.hardware.my_as_ref();

        let module = self.mod_comp.compute(&hardware.mapmap(|f| &f.deviceref), &self.settings.shader_text.my_as_ref(), &self.settings.shader_lang).await.my_as_ref();

        if self.when_send_comp_errs.check_and_update(&[*module.version()]) {
            // TODO: Send compilation results
        }

        let successful_module = module.map(|f| match f {
            Some(Ok(comp)) => Some(comp),
            _ => None
        });

        let blit = self.blitter.compute(&hardware, &self.settings.output_texture_format).await.my_as_ref();


        let rerendered = self.backend
            .step(
                &self.settings.preout_size,
                &self.settings.hardware.my_as_ref(),
                &successful_module,
                entry_point,
                &blit,
                self.render_on_invalid,
                &self.settings.output_texture_view.my_as_ref(),
            )
            .await;

        if rerendered {
            // TODO: Send render notif
        }
    }
}
