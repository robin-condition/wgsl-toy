use egui::{KeyboardShortcut, Modifiers, ViewportCommand, Widget};
use egui_tiles::Tree;

mod tiles_tree_stuff;

use crate::app::{
    egui_shaderwheels_logic::RenderCtx,
    shader_content_manager::{ShaderInfo, ShaderStorageConnection, ShaderStorageConnectionManager},
    tiles_tree_stuff::{create_basic_tree, ShaderWheelsPane, TreeBehavior},
};

mod cfg_pane;
mod editor_gui;
mod egui_shaderwheels_logic;
mod eguice_syntax;
mod error_viewer;
mod shader_content_manager;

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct App {
    #[serde(skip)]
    inf: RenderCtx,

    #[serde(skip)]
    storage_manager: ShaderStorageConnectionManager,

    #[serde(skip)]
    tree: Tree<ShaderWheelsPane>,

    current_shader_inf: ShaderInfo,

    #[serde(skip)]
    compile_on_change: bool,
}

impl Default for App {
    fn default() -> Self {
        Self {
            inf: RenderCtx::default(),
            current_shader_inf: ShaderInfo::default(),
            compile_on_change: true,
            tree: create_basic_tree(),
            storage_manager: ShaderStorageConnectionManager::default(),
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

        //rctx.dep_graph
        rctx.client
            .set_shader_text(state.current_shader_inf.contents.clone());
        //rctx.dep_graph.set_entry_point("main".to_string());

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

        let changed = self
            .storage_manager
            .connection
            .saving_needed(&self.current_shader_inf);
        let viewport_title = if changed {
            self.current_shader_inf.name.clone() + "*"
        } else {
            self.current_shader_inf.name.clone()
        } + " - ShaderWheels";
        ctx.send_viewport_cmd(ViewportCommand::Title(viewport_title));

        let mut saved = false;

        ctx.input_mut(|i| {
            if i.consume_shortcut(&KeyboardShortcut::new(Modifiers::CTRL, egui::Key::S)) {
                saved = true;
            }
        });

        if saved {
            // Saving should happen here!
            self.storage_manager.start_save(&self.current_shader_inf);
        }

        self.storage_manager.update();

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

        egui::CentralPanel::default().show(ctx, |ui| {
            let mut recomp_on_invalid = true; //self.inf.dep_graph.recompute_on_invalidation;
            let mut behav = TreeBehavior {
                rctx: &mut self.inf,
                current_shader_text: &mut self.current_shader_inf.contents,
                compile_on_change: &mut self.compile_on_change,
                recompute_on_invalidate: &mut recomp_on_invalid,
                renderstate: _frame.wgpu_render_state().as_ref().unwrap(),

                shader_text_changed: false,
                recompute_on_textchange_changed: false,
            };
            self.tree.ui(&mut behav, ui);
            let shader_changed = behav.shader_text_changed;
            let recomp_changed = behav.recompute_on_textchange_changed;
            //self.inf.dep_graph.recompute_on_invalidation = recomp_on_invalid;

            if self.compile_on_change && (shader_changed || recomp_changed) {
                self.inf
                    .client
                    .set_shader_text(self.current_shader_inf.contents.clone());
            }
            //Tree::new("tree", root, tiles)
            //egui_shaderwheels_logic::draw(&mut self.inf, ui);
        });
    }
}
