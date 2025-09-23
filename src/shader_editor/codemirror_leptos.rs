use std::time::Duration;

use codemirror_wgsl;
use leptos::{component, ev::keydown, html::Div, logging, prelude::*, view, IntoView};
use leptos_use::{use_document, use_event_listener};

#[component]
pub fn CodeMirrorEditor(
    #[prop(into)] start_text: Signal<String>,
    read_cur_editor_text: ReadSignal<String>,
    set_editor_text: WriteSignal<String>,
    update_every: u32,
    mut on_save: impl FnMut(String) + 'static,
) -> impl IntoView {
    let area_node_ref = NodeRef::<Div>::new();

    let (editor, set_editor) = signal_local(None);
    let editor_exists = move || editor.read().is_some();

    let (timer_handle, set_timer_handle) = signal_local(None);

    let reset_handle = |handle: Option<TimeoutHandle>| match handle {
        None => (),
        Some(handl) => handl.clear(),
    };

    let record_text = move || {
        if let Some(real_editor) = editor.read_untracked().as_ref() {
            let editor_text = codemirror_wgsl::get_editor_text(real_editor);
            if read_cur_editor_text.read_untracked() == editor_text {
                return;
            }
            set_editor_text.set(editor_text);
        }
    };

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
        reset_handle(timer_handle.get_untracked());
        set_timer_handle.set(None);
        if e.ctrl_key() && e.key() == "s" {
            logging::log!("Ctrl + S intercepted, recompiling.");
            e.prevent_default();

            if editor_exists() {
                let text_contents =
                    codemirror_wgsl::get_editor_text(editor.read().as_ref().unwrap());
                set_editor_text.set(text_contents.clone());
                on_save(text_contents);
            }
        } else {
            set_timer_handle.set(Some(
                set_timeout_with_handle(record_text, Duration::from_millis(update_every as u64))
                    .unwrap(),
            ));
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
