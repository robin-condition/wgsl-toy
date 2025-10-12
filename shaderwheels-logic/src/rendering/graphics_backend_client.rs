use std::sync::mpsc::Sender;

use wgpu::TextureView;

use crate::rendering::{
    communication::{BacktalkReceivers, SettingsSenders, create_backtalk_pair, create_pair},
    graphics_backend_worker::{self, Worker},
    shader_config::{GPUAdapterInfo, ShaderConfig},
};

struct LocalSettings {
    preout_size: Option<(u32, u32)>,
    shader_cfg: ShaderConfig,
}

pub struct GraphicsClient {
    // Inputs
    senders: SettingsSenders,
    local_settings: LocalSettings,
    receivers: BacktalkReceivers,
}

impl GraphicsClient {
    pub fn new(shader_cfg: ShaderConfig) -> Self {
        let (sends, recvs) = create_pair();
        let (b_sends, b_recvs) = create_backtalk_pair();

        let worker = Worker::new(recvs, b_sends);
        worker.start_in_background();

        let _ = sends.shader_content.send(shader_cfg.clone());

        GraphicsClient {
            senders: sends,
            local_settings: LocalSettings {
                preout_size: None,
                shader_cfg: shader_cfg,
            },
            receivers: b_recvs,
        }
    }

    pub fn get_preout_size(&self) -> Option<(u32, u32)> {
        self.local_settings.preout_size
    }

    pub fn set_preout_size(&mut self, size: (u32, u32)) {
        if self.local_settings.preout_size != Some(size) {
            self.local_settings.preout_size = Some(size);
            let _ = self.senders.preout_size.send(size);
        }
    }

    pub fn set_output_view(&self, output_view: TextureView) {
        let _ = self.senders.output_texture_view.send(output_view);
    }

    pub fn set_hardware(&self, hw: GPUAdapterInfo) {
        let _ = self.senders.hardware.send(hw);
    }

    pub fn set_shader_text(&mut self, text: String) {
        self.local_settings.shader_cfg.content = text;
        let _ = self
            .senders
            .shader_content
            .send(self.local_settings.shader_cfg.clone());
    }

    pub fn get_should_swap(&mut self) -> bool {
        self.receivers.render_success.try_recv().is_ok()
    }
}

impl Drop for GraphicsClient {
    fn drop(&mut self) {
        let _ = self.senders.kill.send(());
    }
}
