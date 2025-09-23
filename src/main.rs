use leptos::{html::Canvas, logging, prelude::*, task::spawn_local};
use reactive_stores::Store;
use wgpu::{SurfaceConfiguration, SurfaceTarget, util::TextureBlitterBuilder};

use crate::{codemirror_leptos::CodeMirrorEditor, compute_canvas::ComputeCanvas};
pub mod codemirror_leptos;
pub mod compute_canvas;

#[component]
fn App() -> impl IntoView {
    let (count, set_count) = signal(0);

    let (text, set_text) = signal(include_str!("compute.wgsl").to_owned());

    Effect::new(move || {
        leptos::logging::log!("shader: {:?}", text.get());
    });

    view! {
        <ComputeCanvas size=(500u32, 500u32) shader_text=text/>
        <CodeMirrorEditor set_text/>
    }
}

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(|| App())
}
