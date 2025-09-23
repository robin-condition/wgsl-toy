use leptos::prelude::*;

use crate::shader_editor::ShaderEditor;
pub mod codemirror_leptos;
pub mod compute_canvas;
pub mod shader_editor;

#[component]
fn App() -> impl IntoView {
    view! { <ShaderEditor /> }
}

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(|| App())
}
