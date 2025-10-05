use eframe::egui_wgpu::RenderState;
use egui::{pos2, Color32, KeyboardShortcut, Layout, Modifiers, Rect, RichText, TextureId, Widget};
use egui_code_editor::ColorTheme;
use shaderwheels_logic::rendering::{
    CompleteGraphicsDependencyGraph, CompleteGraphicsInitialConfig, GPUAdapterInfo,
};
use wgpu::{wgt::TextureDescriptor, Extent3d, TextureFormat};

use crate::app::{egui_shaderwheels_logic::RenderCtx, eguice_syntax::wgsl_syntax};

mod egui_shaderwheels_logic;
mod eguice_syntax;

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct App {
    #[serde(skip)]
    inf: RenderCtx,

    current_shader_text: String,

    #[serde(skip)]
    compile_on_change: bool,

    #[serde(skip)]
    recompute_on_invalidate: bool,
}

impl Default for App {
    fn default() -> Self {
        Self {
            inf: RenderCtx::default(),
            current_shader_text: shaderwheels_logic::rendering::DEFAULT_COMPUTE.to_string(),
            compile_on_change: false,
            recompute_on_invalidate: false,
        }
    }
}

impl App {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.

        let state: App = if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            Default::default()
        };

        let mut rctx = egui_shaderwheels_logic::onetime_hardware_setup(cc);

        rctx.dep_graph
            .set_shader_text(state.current_shader_text.clone());
        rctx.dep_graph.set_entry_point("main".to_string());

        Self { inf: rctx, ..state }
    }
}

struct EguiCodeEditorWrapperWidget<'a> {
    edit: egui_code_editor::CodeEditor,
    text: &'a mut String,
}

impl<'a> Widget for EguiCodeEditorWrapperWidget<'a> {
    fn ui(mut self, ui: &mut egui::Ui) -> egui::Response {
        self.edit.show(ui, self.text).response
    }
}

fn egui_code_editor_to_widget(
    edit: egui_code_editor::CodeEditor,
    text: &mut String,
) -> impl Widget {
    EguiCodeEditorWrapperWidget { edit, text }
}

impl eframe::App for App {
    /// Called by the framework to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        let mut saved = false;

        ctx.input_mut(|i| {
            if i.consume_shortcut(&KeyboardShortcut::new(Modifiers::CTRL, egui::Key::S)) {
                saved = true;
            }
        });

        if saved {
            // Saving should happen here!
        }

        _frame.wgpu_render_state().unwrap();

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::MenuBar::new().ui(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(16.0);
                }

                egui::widgets::global_theme_preference_buttons(ui);
            });
        });

        egui::SidePanel::right("editor_panel").show(ctx, |ui| {
            //let target_size = ui.available_size();

            let changed = //ui .add_sized(ui.available_size(),*/
            ui.add(
                    egui_code_editor_to_widget(
                        egui_code_editor::CodeEditor::default()
                            .id_source("editor!")
                            .with_theme(ColorTheme::GRUVBOX)
                            .with_syntax(wgsl_syntax())
                            .with_numlines(true)
                            .with_rows(50)
                            .with_fontsize(14f32),
                            &mut self.current_shader_text
                    )
                )
                .changed();

            let rou_changed = ui
                .checkbox(&mut self.compile_on_change, "Recompile on text change")
                .changed();

            let roi_changed = ui
                .checkbox(&mut self.recompute_on_invalidate, "Recompute on recompile")
                .changed();

            ui.with_layout(Layout::bottom_up(egui::Align::Min), |bottom_ui| {
                let rt = if let Some(err) = self.inf.dep_graph.get_compilation_error() {
                    let err_text = match err {
                        wgpu::Error::OutOfMemory { source: _ } => "",
                        wgpu::Error::Validation {
                            source: _,
                            description,
                        } => &description,
                        wgpu::Error::Internal {
                            source: _,
                            description,
                        } => &description,
                    };
                    RichText::new(err_text).color(Color32::RED)
                } else {
                    RichText::new("Latest compilation successful.")
                };

                bottom_ui.label(rt.size(14f32));
            });

            if roi_changed {
                self.inf.dep_graph.recompute_on_invalidation = self.recompute_on_invalidate;
            }

            if (changed || rou_changed) && self.compile_on_change {
                self.inf
                    .dep_graph
                    .set_shader_text(self.current_shader_text.clone());
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            egui_shaderwheels_logic::draw(&mut self.inf, ui);
        });
    }
}
