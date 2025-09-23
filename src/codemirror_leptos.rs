use std::rc::Rc;

//use codemirror::{DocApi, Editor, EditorOptions};
//use monaco::api::{CodeEditor, CodeEditorOptions, TextModel};
use leptos::{component, html::Div, prelude::*, view, IntoView};
use codemirror_wgsl;


#[component]
pub fn CodeMirrorEditor() -> impl IntoView {
    let area_node_ref = NodeRef::<Div>::new();

    let (text, set_text) = signal("".to_owned());

    let (editor, set_editor) = signal_local(None);
    let editor_exists = move || editor.read().is_some();

    Effect::new(move || {
        if editor_exists() {
            return;
        }
        if let Some(textarea_node) = area_node_ref.get() {
            
            //let options = EditorOptions::default().line_numbers(true);
            //let editor = Editor::from_text_area(&textarea_node, &options);

            //editor.set_value("fn hello() { \n }");

            //let editor = CodeEditor::create(&textarea_node, Some(CodeEditorOptions::default()));
            //editor.set_model(&TextModel::create("fn hello() { \n }", None, None).unwrap());

            set_editor.set(Some(codemirror_wgsl::make_wgsl_editor(&textarea_node)));
        }
    });

    view!{
        <div>
        <button on:click= move |_| {
                if editor_exists() {
                    set_text.set(codemirror_wgsl::get_editor_text(editor.read().as_ref().unwrap()));
                    leptos::logging::log!("mytext!: {:?}", text.get());
                }
                
            }>Recompile!</button>
            <div node_ref=area_node_ref></div>
        </div>
    }
}