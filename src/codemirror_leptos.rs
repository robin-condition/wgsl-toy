use codemirror::{DocApi, Editor, EditorOptions};
//use monaco::api::{CodeEditor, CodeEditorOptions, TextModel};
use leptos::{component, html::{Div, Textarea}, prelude::{Effect, Get, NodeRef, NodeRefAttribute}, view, IntoView};


#[component]
pub fn CodeMirrorEditor() -> impl IntoView {
    let area_node_ref = NodeRef::<Textarea>::new();

    Effect::new(move || {
        if let Some(textarea_node) = area_node_ref.get() {
            
            let options = EditorOptions::default().line_numbers(true);
            let editor = Editor::from_text_area(&textarea_node, &options);

            editor.set_value("fn hello() { \n }");

            //let editor = CodeEditor::create(&textarea_node, Some(CodeEditorOptions::default()));
            //editor.set_model(&TextModel::create("fn hello() { \n }", None, None).unwrap());
        }
    });

    view!{
        <textarea node_ref=area_node_ref></textarea>
    }
}