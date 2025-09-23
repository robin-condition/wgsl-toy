use wasm_bindgen::prelude::*;
use web_sys::HtmlElement;

#[wasm_bindgen(module = "/src/package.js")]
extern "C" {

    pub type EditorView;

    #[wasm_bindgen]
    fn MakeWgslEditor(parentComponent: &HtmlElement, s: &str) -> EditorView;

    #[wasm_bindgen]
    fn GetEditorText(editor: &EditorView) -> String;

    #[wasm_bindgen]
    fn SetEditorText(editor: &EditorView, s: &str);
}

#[wasm_bindgen]
pub fn make_wgsl_editor(parent_component: &HtmlElement, contents: &str) -> EditorView {
    return MakeWgslEditor(parent_component, contents);
}

#[wasm_bindgen]
pub fn get_editor_text(editor: &EditorView) -> String {
    return GetEditorText(editor);
}

#[wasm_bindgen]
pub fn set_editor_text(editor: &EditorView, contents: &str) {
    SetEditorText(editor, contents);
}
