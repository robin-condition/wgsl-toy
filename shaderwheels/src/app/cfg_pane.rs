use egui::Ui;

pub fn add_transient_cfg_pane(
    compile_on_change: &mut bool,
    recompute_on_invalidate: &mut bool,
    recompile_on_textchange_changed: &mut bool,
    ui: &mut Ui,
) {
    *recompile_on_textchange_changed = ui
        .checkbox(compile_on_change, "Recompile on text change")
        .changed();

    ui.checkbox(recompute_on_invalidate, "Recompute on recompile");
}
