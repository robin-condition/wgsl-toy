use leptos::{logging, prelude::*};

use crate::shader_editor::{
    codemirror_leptos::CodeMirrorEditor,
    compute_canvas::ComputeCanvas,
    gpu_records::{GPUPrepState, ShaderCallPrep},
};

pub mod codemirror_leptos;
pub mod compute_canvas;
pub mod gpu_records;

fn prepare_pipeline_signal(
    gpu_prep: ReadSignal<Option<GPUPrepState<'static>>, LocalStorage>,
    shader_text: Signal<String>,
) -> ReadSignal<Option<ShaderCallPrep>, LocalStorage> {
    let (pipeline_prep, set_pipeline_prep) = signal_local(None);
    Effect::new(move || {
        if let Some(gpu) = gpu_prep.read().as_ref() {
            logging::log!("Recompiling shader!");
            set_pipeline_prep.set(Some(gpu_records::prep_shader(gpu, shader_text.get())));
        }
    });
    pipeline_prep
}

#[component]
pub fn ShaderEditor() -> impl IntoView {
    let (starting_text, _set_starting_text) = signal(include_str!("compute.wgsl").to_owned());

    let size = (500u32, 500u32);

    let (gpu_prep, set_gpu_prep) = signal_local(None);

    let (editor_text, set_editor_text) = signal(starting_text.get());

    let (shader_text, set_shader_text) = signal(starting_text.get());

    let pipeline_prep = prepare_pipeline_signal(gpu_prep, shader_text.into());

    Effect::new(move || {
        if gpu_prep.read().is_some() && pipeline_prep.read().is_some() {
            logging::log!("Re shading!");
            gpu_records::do_shader(
                gpu_prep.read().as_ref().unwrap(),
                pipeline_prep.read().as_ref().unwrap(),
            );
        }
    });

    Effect::new(move || {
        let _ = editor_text.read();
        set_shader_text.set(editor_text.get());
        logging::log!("Updated!");
    });

    view! {
        <ComputeCanvas size set_prep_state=set_gpu_prep />
        <CodeMirrorEditor
            start_text=starting_text
            get_editor_text=()
            set_editor_text=set_editor_text
            on_save=move |_| {
                logging::log!("On save callback!");
            }
        />
    }
}
