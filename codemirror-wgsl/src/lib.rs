use wasm_bindgen::prelude::*;
use web_sys::HtmlElement;

#[wasm_bindgen(module = "/src/package.js")]
extern "C" {

    pub type EditorView;

    #[wasm_bindgen]
    fn MakeWgslEditor(parentComponent: &HtmlElement) -> EditorView;

    #[wasm_bindgen]
    fn GetEditorText(editor: &EditorView) -> String;
}

#[wasm_bindgen]
pub fn make_wgsl_editor(parent_component: &HtmlElement) -> EditorView {
    return MakeWgslEditor(parent_component);
}

#[wasm_bindgen]
pub fn get_editor_text(editor: &EditorView) -> String {
    return GetEditorText(editor);
}