use egui::{Color32, RichText, Ui};

use crate::app::egui_shaderwheels_logic::RenderCtx;

pub fn add_error_viewer(rctx: &RenderCtx, ui: &mut Ui) {
    let rt = RichText::new("No support for now"); /*if let Some(err) = rctx.dep_graph.get_compilation_error() {
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
                                                  };*/

    ui.label(rt.size(14f32));
}
