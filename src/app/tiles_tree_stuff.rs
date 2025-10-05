use eframe::egui_wgpu::RenderState;
use egui_tiles::{Behavior, UiResponse};

use crate::app::{
    editor_gui::add_editor,
    egui_shaderwheels_logic::{self, RenderCtx},
    error_viewer::add_error_viewer,
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PaneType {
    CodeEditor,
    ErrorViewer,
    RenderTarget,
}

impl PaneType {
    pub fn name(&self) -> &'static str {
        match self {
            PaneType::CodeEditor => "Editor",
            PaneType::ErrorViewer => "Error Viewer",
            PaneType::RenderTarget => "Render Viewer",
        }
    }
}

pub struct ShaderWheelsPane {
    id_number: usize,
    kind: PaneType,
}

// egui tiles stuff
// Based on https://github.com/rerun-io/egui_tiles/blob/main/examples/simple.rs
pub fn create_basic_tree() -> egui_tiles::Tree<ShaderWheelsPane> {
    let mut next_view_nr = 0;
    let mut gen_pane = |k| {
        let pane = ShaderWheelsPane {
            id_number: next_view_nr,
            kind: k,
        };
        next_view_nr += 1;
        pane
    };

    let mut tiles = egui_tiles::Tiles::default();

    let render_pane = gen_pane(PaneType::RenderTarget);
    let editor_pane = gen_pane(PaneType::CodeEditor);
    let error_pane = gen_pane(PaneType::ErrorViewer);

    let right_half = {
        let edit = tiles.insert_pane(editor_pane);
        let error = tiles.insert_pane(error_pane);
        tiles.insert_vertical_tile(vec![edit, error])
    };

    let left_half = tiles.insert_pane(render_pane);

    let root = tiles.insert_horizontal_tile(vec![left_half, right_half]);

    egui_tiles::Tree::new("my_tree", root, tiles)
}

// Freehanding this
// Nvm gave up and referred to example again
pub struct TreeBehavior<'a> {
    pub rctx: &'a mut RenderCtx,
    pub current_shader_text: &'a mut String,
    pub compile_on_change: &'a mut bool,
    pub recompute_on_invalidate: &'a mut bool,
    pub renderstate: &'a RenderState,
}

impl<'a> Behavior<ShaderWheelsPane> for TreeBehavior<'a> {
    fn pane_ui(
        &mut self,
        ui: &mut egui::Ui,
        _tile_id: egui_tiles::TileId,
        pane: &mut ShaderWheelsPane,
    ) -> egui_tiles::UiResponse {
        let drag_rect = match pane.kind {
            PaneType::CodeEditor => {
                let lab = ui.label("I'm an editor");
                add_editor(
                    self.rctx,
                    self.current_shader_text,
                    self.compile_on_change,
                    self.recompute_on_invalidate,
                    ui,
                );
                lab
            }
            PaneType::ErrorViewer => {
                let lab = ui.label("I'm an error viewer");
                add_error_viewer(&self.rctx, ui);
                lab
            }
            PaneType::RenderTarget => {
                let lab = ui.label("I'm a render target");
                egui_shaderwheels_logic::draw(self.rctx, self.renderstate, ui);
                lab
            }
        }
        .rect;

        let dragged = ui
            .allocate_rect(drag_rect, egui::Sense::DRAG)
            .on_hover_cursor(egui::CursorIcon::Grab)
            .dragged();
        if dragged {
            UiResponse::DragStarted
        } else {
            UiResponse::None
        }
    }

    fn tab_title_for_pane(&mut self, pane: &ShaderWheelsPane) -> egui::WidgetText {
        pane.kind.name().into()
    }
}
