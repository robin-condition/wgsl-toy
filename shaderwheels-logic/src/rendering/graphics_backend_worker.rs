use std::sync::mpsc::Receiver;

use cardigan_incremental::{ReceivedVersioned, Versioned, VersionedInputs, memoized};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::wasm_bindgen::prelude::Closure;
use wgpu::{TextureFormat, TextureView};

use crate::rendering::{
    communication::{BacktalkSenders, SettingsReceivers},
    graphics_backend_worker::{
        compute_worker::ComputeWorkerPart,
        fragment_worker::FragmentWorkerPart,
        shared::{blitter, module_comp, BackendWorker},
    },
    shader_config::{GPUAdapterInfo, ShaderConfig, ShaderLanguage}, WGSL_ENTRY,
};

mod compute_worker;
mod fragment_worker;
mod shared;

enum ArbitraryWorker {
    ComputeWorker(ComputeWorkerPart),
    FragmentWorker(FragmentWorkerPart),
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
        output_view: &Option<&TextureView>,
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
            ArbitraryWorker::FragmentWorker(fragment_worker_part) => {
                fragment_worker_part
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
        }
    }
}

fn latest_from_receiver<T>(recvr: &Receiver<T>) -> Option<T> {
    if let Ok(mut val) = recvr.try_recv() {
        while let Ok(new_val) = recvr.try_recv() {
            val = new_val;
        }

        return Some(val);
    }
    None
}

#[derive(Default)]
pub struct VersionedSettings {
    pub shader_text: Versioned<String>,
    pub append_environment: Versioned<bool>,
    pub shader_lang: Versioned<ShaderLanguage>,
    pub preout_size: Versioned<(u32, u32)>,
    pub hardware: Versioned<GPUAdapterInfo>,
    pub output_texture_view: Option<TextureView>,
    pub output_texture_format: Versioned<TextureFormat>,
}

pub struct Worker {
    settings_recvrs: SettingsReceivers,
    backtalk_senders: BacktalkSenders,
    settings: VersionedSettings,
    backend: ArbitraryWorker,

    render_on_invalid: bool,

    blitter: blitter,
    mod_comp: module_comp,
    when_send_comp_errs: VersionedInputs<1>,
}

impl Worker {
    pub fn new(recvs: SettingsReceivers, sends: BacktalkSenders) -> Self {
        Self {
            settings_recvrs: recvs,
            backtalk_senders: sends,
            settings: VersionedSettings::default(),
            backend: //ArbitraryWorker::ComputeWorker(ComputeWorkerPart::default()),
            ArbitraryWorker::FragmentWorker(FragmentWorkerPart::default()),
            render_on_invalid: true,
            blitter: Default::default(),
            mod_comp: Default::default(),
            when_send_comp_errs: Default::default(),
        }
    }

    fn read_recvrs(&mut self) {
        if let Some(cfg) = latest_from_receiver(&self.settings_recvrs.shader_content) {
            self.settings.shader_lang.set_to_next(Some(cfg.language));
            self.settings
                .shader_text
                .set_to_next_if_unequal(Some(cfg.content));
            // TODO: Update backend and append_env
        }

        if let Some(hw) = latest_from_receiver(&self.settings_recvrs.hardware) {
            self.settings.hardware.set_to_next(Some(hw));
        }

        if let Some(out_view) = latest_from_receiver(&self.settings_recvrs.output_texture_view) {
            self.settings.output_texture_view = Some(out_view);
        }

        if let Some(preout_size) = latest_from_receiver(&self.settings_recvrs.preout_size) {
            self.settings
                .preout_size
                .set_to_next_if_unequal(Some(preout_size));
        }

        self.settings.output_texture_format =
            Versioned::default().next(Some(TextureFormat::Rgba8Unorm));
    }

    async fn longrunning_task(mut self) {
        loop {
            Self::pacing_fn().await;

            if let Ok(()) = self.settings_recvrs.kill.try_recv() {
                return;
            }
            self.step().await
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn start_in_background(self) {
        wasm_bindgen_futures::spawn_local(self.longrunning_task());
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn start_in_background(mut self) {
        std::thread::spawn(move || {
            pollster::block_on(self.longrunning_task());
        });
    }

    #[cfg(target_arch = "wasm32")]
    pub async fn pacing_fn() {
        use std::time::Duration;

        async_std::task::sleep(Duration::from_millis(16u64)).await;
        //let _ = wasm_timer::Delay::new(Duration::from_millis(16u64)).await;
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub async fn pacing_fn() {}

    async fn step(&mut self) {
        self.read_recvrs();

        let hardware = self.settings.hardware.my_as_ref();

        let module = self
            .mod_comp
            .compute(
                &hardware.mapmap(|f| &f.deviceref),
                &self.settings.shader_text.my_as_ref(),
                &self.settings.shader_lang,
            )
            .await
            .my_as_ref();

        if self
            .when_send_comp_errs
            .check_and_update(&[*module.version()])
        {
            // TODO: Send compilation results
        }

        let successful_module = module.map(|f| match f {
            Some(Ok(comp)) => Some(comp),
            _ => None,
        });

        let blit = self
            .blitter
            .compute(&hardware, &self.settings.output_texture_format)
            .await
            .my_as_ref();

        let rerendered = self
            .backend
            .step(
                &self.settings.preout_size,
                &self.settings.hardware.my_as_ref(),
                &successful_module,
                &Versioned::default().next(Some(&WGSL_ENTRY.to_string())),
                &blit,
                self.render_on_invalid,
                &self.settings.output_texture_view.as_ref(),
            )
            .await;

        if rerendered {
            // TODO: Send render notif
            self.settings.output_texture_view = None;
            let _ = self.backtalk_senders.render_success.send(());
        }
    }
}
