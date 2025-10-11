use crate::rendering::shader_config::{GPUAdapterInfo, ShaderConfig};
use std::sync::mpsc::{self, Receiver, Sender};
use wgpu::TextureView;

macro_rules! channel_side {
    ($name:ident, $kind:ident) => {
        pub struct $name {
            pub shader_content: $kind<ShaderConfig>,

            pub hardware: $kind<GPUAdapterInfo>,
            pub output_texture_view: $kind<TextureView>,
            pub preout_size: $kind<(u32, u32)>,

            pub kill: $kind<()>,
        }
    };
}
channel_side! {SettingsReceivers, Receiver}
channel_side! {SettingsSenders, Sender}

pub fn create_pair() -> (SettingsSenders, SettingsReceivers) {
    let (cfg_send, cfg_receive) = mpsc::channel::<ShaderConfig>();
    let (hardware_send, hardware_receive) = mpsc::channel::<GPUAdapterInfo>();
    let (output_tex_view_send, output_tex_view_receive) = mpsc::channel::<TextureView>();
    let (preout_size_send, preout_size_receive) = mpsc::channel::<(u32, u32)>();
    let (kill_send, kill_receive) = mpsc::channel::<()>();

    (
        SettingsSenders {
            shader_content: cfg_send,
            hardware: hardware_send,
            output_texture_view: output_tex_view_send,
            preout_size: preout_size_send,
            kill: kill_send,
        },
        SettingsReceivers {
            shader_content: cfg_receive,
            hardware: hardware_receive,
            output_texture_view: output_tex_view_receive,
            preout_size: preout_size_receive,
            kill: kill_receive,
        },
    )
}

macro_rules! backtalk_side {
    ($name:ident, $kind:ident) => {
        pub struct $name {
            pub render_success: $kind<()>,
        }
    };
}
backtalk_side! {BacktalkReceivers, Receiver}
backtalk_side! {BacktalkSenders, Sender}

pub fn create_backtalk_pair() -> (BacktalkSenders, BacktalkReceivers) {
    let (render_send, render_recv) = mpsc::channel::<()>();

    (
        BacktalkSenders {
            render_success: render_send,
        },
        BacktalkReceivers {
            render_success: render_recv,
        },
    )
}
