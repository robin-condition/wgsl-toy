use codemirror_wgsl;
use leptos::{IntoView, component, ev::keydown, html::Div, logging, prelude::*, view};
use leptos_use::{use_document, use_event_listener};

#[component]
pub fn CodeMirrorEditor(
    #[prop(into)] start_text: Signal<String>,
    #[prop(into)] get_editor_text: Signal<()>,
    set_editor_text: WriteSignal<String>,
    mut on_save: impl FnMut(String) + 'static,
) -> impl IntoView {
    let area_node_ref = NodeRef::<Div>::new();

    let (editor, set_editor) = signal_local(None);
    let editor_exists = move || editor.read().is_some();

    Effect::new(move || {
        if let Some(real_editor) = editor.read_untracked().as_ref() {
            let _ = get_editor_text.get();
            let editor_text = codemirror_wgsl::get_editor_text(real_editor);
            set_editor_text.set(editor_text);
        }
    });

    // Respond to assignments of starting text.
    Effect::new(move || {
        if !editor_exists() {
            return;
        }
        logging::log!("SETTING EDITOR TEXT");
        codemirror_wgsl::set_editor_text(
            editor.read().as_ref().unwrap(),
            start_text.read().as_ref(),
        );
        set_editor_text.set(start_text.get());
    });

    Effect::new(move || {
        if editor_exists() {
            return;
        }
        if let Some(textarea_node) = area_node_ref.get() {
            set_editor.set(Some(codemirror_wgsl::make_wgsl_editor(
                &textarea_node,
                start_text.read().as_str(),
            )));
        }
    });

    let _ = use_event_listener(use_document(), keydown, move |e| {
        if e.ctrl_key() && e.key() == "s" {
            logging::log!("Ctrl + S intercepted, recompiling.");
            e.prevent_default();

            if editor_exists() {
                let text_contents =
                    codemirror_wgsl::get_editor_text(editor.read().as_ref().unwrap());
                set_editor_text.set(text_contents.clone());
                on_save(text_contents);
            }
        }
    });

    view! {
        <div>
            <button on:click=move |_| {
                if editor_exists() {
                    set_editor_text
                        .set(codemirror_wgsl::get_editor_text(editor.read().as_ref().unwrap()));
                }
            }>Recompile!</button>
            <div node_ref=area_node_ref></div>
        </div>
    }
}
