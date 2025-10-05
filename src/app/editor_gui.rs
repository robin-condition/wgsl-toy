use egui::Ui;
use egui_code_editor::ColorTheme;

use crate::app::{
    egui_code_editor_to_widget, egui_shaderwheels_logic::RenderCtx, eguice_syntax::wgsl_syntax,
};

pub fn add_editor(
    rctx: &mut RenderCtx,
    current_shader_text: &mut String,
    compile_on_change: &mut bool,
    recompute_on_invalidate: &mut bool,
    ui: &mut Ui,
) {
    let changed = //ui .add_sized(ui.available_size(),*/
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

    let rou_changed = ui
        .checkbox(compile_on_change, "Recompile on text change")
        .changed();

    let roi_changed = ui
        .checkbox(recompute_on_invalidate, "Recompute on recompile")
        .changed();

    if roi_changed {
        rctx.dep_graph.recompute_on_invalidation = *recompute_on_invalidate;
    }

    if (changed || rou_changed) && *compile_on_change {
        rctx.dep_graph.set_shader_text(current_shader_text.clone());
    }
}
