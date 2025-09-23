import {wgsl} from '@iizukak/codemirror-lang-wgsl';
import { EditorView, basicSetup } from 'codemirror';
import { EditorState } from '@codemirror/state';

export function MakeWgslEditor(parentComponent, s) {
    return new EditorView({
        extensions: [basicSetup, wgsl()],
        parent: parentComponent,
        doc: s
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

export function SetEditorText(editor, s) {
    editor.setState(EditorState.create({
        extensions: [basicSetup, wgsl()],
        doc: s
    }));
}