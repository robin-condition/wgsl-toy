use leptos::prelude::*;

use crate::{codemirror_leptos::CodeMirrorEditor, compute_canvas::ComputeCanvas};
pub mod codemirror_leptos;
pub mod compute_canvas;

#[component]
fn App() -> impl IntoView {
    let starting_text = include_str!("compute.wgsl").to_owned();

    let (text, set_text) = signal(starting_text.clone());

    view! {
        <ComputeCanvas size=(500u32, 500u32) shader_text=text />
        <CodeMirrorEditor start_text=starting_text set_text />
    }
}

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(|| App())
}
