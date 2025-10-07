use egui::Ui;
use egui_code_editor::ColorTheme;

use crate::app::{egui_code_editor_to_widget, eguice_syntax::wgsl_syntax};

pub fn add_editor(current_shader_text: &mut String, changed: &mut bool, ui: &mut Ui) {
    *changed = //ui .add_sized(ui.available_size(),*/
            ui.add(
                    egui_code_editor_to_widget(
                        egui_code_editor::CodeEditor::default()
                            .id_source("editor!")
                            .with_theme(ColorTheme::GRUVBOX)
                            .with_syntax(wgsl_syntax())
                            .with_numlines(true)
                            //.with_rows(50)
                            .with_fontsize(14f32),
                            current_shader_text
                    )
                )
                .changed();
}
