use leptos::{html::Canvas, logging, prelude::*};

use crate::shader_editor::gpu_records::{self, GPUAdapterPrep, PipelinePrep};

pub type GPUAdapterPrepReactive = LocalResource<Result<GPUAdapterPrep<'static>, ()>>;
pub type PipelinePrepReactive = LocalResource<Result<PipelinePrep, ()>>;

pub fn prepare_pipeline_signal(
    gpu_prep: GPUAdapterPrepReactive,// Signal<Option<Result<GPUAdapterPrep<'static>, ()>>, LocalStorage>,
    shader_text: Signal<String>,
) -> PipelinePrepReactive {
    LocalResource::new(move || async move {
        if let Some(Ok(gpu)) = gpu_prep.read().as_ref() {
            logging::log!("Recompiling shader!");
            return Ok(gpu_records::prep_shader(gpu, shader_text.get()));
        }
        return Err(());
    })
}

pub fn prepare_shader_effect(
    gpu_adapter_prep: GPUAdapterPrepReactive,
    pipeline_prep: PipelinePrepReactive,
) {
    Effect::new(move || {
        if let (Some(Ok(gpu)), Some(Ok(pipeline))) = (gpu_adapter_prep.read().as_ref(), pipeline_prep.read().as_ref()) {
            logging::log!("Re shading!");
            gpu_records::do_shader(
                gpu,
                pipeline,
            );
        }
    });
}

pub fn prepare_adapter(canvas_ref: NodeRef<Canvas>, size: Signal<(u32, u32)>)
-> GPUAdapterPrepReactive {
    LocalResource::new(
        move || { async move {
            if canvas_ref.get().is_some() {
                let node = canvas_ref.get().unwrap();
                logging::log!("Doing GPU prep!");
                return Ok(gpu_records::prep_wgpu(node, size.get()).await);
                // https://github.com/gfx-rs/wgpu/blob/trunk/examples/standalone/02_hello_window/src/main.rs
            }
            return Err(());
        } },
    )
}
