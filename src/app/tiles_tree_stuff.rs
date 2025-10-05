use egui::Sense;
use egui_tiles::{Behavior, UiResponse};

use crate::app::egui_shaderwheels_logic::RenderCtx;

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
}

impl<'a> Behavior<ShaderWheelsPane> for TreeBehavior<'a> {
    fn pane_ui(
        &mut self,
        ui: &mut egui::Ui,
        _tile_id: egui_tiles::TileId,
        pane: &mut ShaderWheelsPane,
    ) -> egui_tiles::UiResponse {
        match pane.kind {
            PaneType::CodeEditor => {
                ui.label("I'm an editor");
            }
            PaneType::ErrorViewer => {
                ui.label("I'm an error viewer");
            }
            PaneType::RenderTarget => {
                ui.label("I'm a render target");
            }
        }

        let dragged = ui
            .allocate_rect(ui.max_rect(), egui::Sense::DRAG)
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
