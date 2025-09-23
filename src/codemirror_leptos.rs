use codemirror_wgsl;
use leptos::{
    IntoView, component,
    ev::{EventDescriptor, keydown},
    html::Div,
    logging,
    prelude::*,
    view,
};
use leptos_use::{use_document, use_event_listener};

#[component]
pub fn CodeMirrorEditor(start_text: String, set_text: WriteSignal<String>) -> impl IntoView {
    let area_node_ref = NodeRef::<Div>::new();

    let (editor, set_editor) = signal_local(None);
    let editor_exists = move || editor.read().is_some();

    Effect::new(move || {
        if editor_exists() {
            return;
        }
        if let Some(textarea_node) = area_node_ref.get() {
            set_editor.set(Some(codemirror_wgsl::make_wgsl_editor(
                &textarea_node,
                start_text.as_str(),
            )));
        }
    });

    let _ = use_event_listener(use_document(), keydown, move |e| {
        if e.ctrl_key() && e.key() == "s" {
            logging::log!("Ctrl + S intercepted, recompiling.");
            e.prevent_default();

            if editor_exists() {
                set_text.set(codemirror_wgsl::get_editor_text(
                    editor.read().as_ref().unwrap(),
                ));
            }
        }
    });

    view! {
        <div>
        <button on:click= move |_| {
                if editor_exists() {
                    set_text.set(codemirror_wgsl::get_editor_text(editor.read().as_ref().unwrap()));
                }

            }>Recompile!</button>
            <div node_ref=area_node_ref></div>
        </div>
    }
}
