use leptos::{html::Canvas, prelude::*};

#[component]
pub fn ComputeCanvas(
    #[prop(into)] size: Signal<(u32, u32)>,
    canvas_ref: NodeRef<Canvas>,
) -> impl IntoView {
    view! {
        <canvas width=move || size.get().0 height=move || size.get().1 node_ref=canvas_ref></canvas>
    }
}
