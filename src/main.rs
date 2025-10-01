use leptos::{logging, prelude::*};
use leptos_router::{
    components::{Route, Router, Routes},
    hooks::use_query,
    params::Params,
    path,
};

use crate::shader_editors::ShaderEditorFromId;

pub mod shader_editor;
pub mod shader_editors;

#[component]
fn App() -> impl IntoView {
    // https://github.com/ramosbugs/openidconnect-rs/blob/main/examples/google.rs

    view! {
        <Router>
            <Routes fallback=|| {
                view! { "Routing was unsuccessful" }
            }>
                <Route path=path!(":id") view=ShaderEditorFromId />
            </Routes>
        </Router>
    }
}

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(|| App())
}
