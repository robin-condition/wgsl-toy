use leptos::prelude::*;

use crate::{codemirror_leptos::CodeMirrorEditor, compute_canvas::ComputeCanvas};

#[component]
pub fn ShaderEditor() -> impl IntoView {
    let starting_text = include_str!("compute.wgsl").to_owned();

    let (text, set_text) = signal(starting_text.clone());

    view! {
        <ComputeCanvas size=(500u32, 500u32) shader_text=text />
        <CodeMirrorEditor start_text=starting_text set_text />
    }
}
