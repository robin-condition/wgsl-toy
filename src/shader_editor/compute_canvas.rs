use leptos::{html::Canvas, logging, prelude::*, reactive::spawn_local};

use crate::shader_editor::gpu_records::{GPUPrepState, prep_wgpu};

#[component]
pub fn ComputeCanvas(
    #[prop(into)] size: Signal<(u32, u32)>,
    set_prep_state: WriteSignal<Option<GPUPrepState<'static>>, LocalStorage>,
) -> impl IntoView {
    let node_ref = NodeRef::<Canvas>::new();
    let canvas_exists = move || node_ref.get().is_some();

    Effect::new(move |_| {
        if canvas_exists() {
            let node = node_ref.get().unwrap();
            logging::log!("Doing GPU prep!");
            spawn_local(async move {
                set_prep_state.set(Some(prep_wgpu(node, size.get()).await));
            });
            // https://github.com/gfx-rs/wgpu/blob/trunk/examples/standalone/02_hello_window/src/main.rs
        }
    });

    view! { <canvas width=move || size.get().0 height=move || size.get().1 node_ref=node_ref></canvas> }
}
