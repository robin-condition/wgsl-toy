use leptos::{html::Canvas, logging, prelude::*};

use crate::shader_editor::{codemirror_leptos::CodeMirrorEditor, compute_canvas::ComputeCanvas};

pub mod codemirror_leptos;
pub mod compute_canvas;
pub mod gpu_records;
pub mod reactive_gpu;

pub const DEFAULT_COMPUTE: &str = include_str!("compute.wgsl");

#[component]
pub fn ShaderEditor(#[prop(into)] starting_text: Signal<String>) -> impl IntoView {
    //let (starting_text, _set_starting_text) = signal(DEFAULT_COMPUTE.to_string());

    let (size, set_size) = signal((500u32, 500u32));

    let canvas_ref = NodeRef::<Canvas>::new();

    let gpu_prep = reactive_gpu::prepare_adapter(canvas_ref, size.into());

    let (editor_text, set_editor_text) = signal(starting_text.get());

    let (shader_text, set_shader_text) = signal(starting_text.get());

    let pipeline_prep = reactive_gpu::prepare_pipeline_signal(gpu_prep, shader_text.into());

    reactive_gpu::prepare_shader_effect(gpu_prep, pipeline_prep);

    Effect::new(move || {
        set_shader_text.set(editor_text.get());
        logging::log!("Updated!");
    });

    view! {
        <ComputeCanvas size canvas_ref />
        <CodeMirrorEditor
            start_text=starting_text
            get_editor_text=Trigger::new()
            set_editor_text=set_editor_text
            on_save=move |_| {
                logging::log!("On save callback!");
            }
        />
    }
}
