import {wgsl} from '@iizukak/codemirror-lang-wgsl';
import { EditorView, basicSetup } from 'codemirror';


export function MakeWgslEditor(parentComponent) {
    return new EditorView({
        extensions: [basicSetup, wgsl()],
        parent: parentComponent,
        doc: "fn main() {\n hi;\n}"
    })
}

/**
 * 
 * @param {EditorView} editor 
 * @returns 
 */
export function GetEditorText(editor) {
    return editor.state.doc.toString();
}