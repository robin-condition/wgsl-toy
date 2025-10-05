use egui::{
    pos2, Color32, KeyboardShortcut, Layout, Modifiers, Rect, RichText, Stroke, TextureId, Widget,
};
use egui_code_editor::{ColorTheme, Syntax};
use shaderwheels_logic::rendering::{
    CompleteGraphicsDependencyGraph, CompleteGraphicsInitialConfig, GPUAdapterInfo,
};
use wgpu::{Extent3d, TextureFormat};

use crate::app::eguice_syntax::wgsl_syntax;

mod eguice_syntax;

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct App {
    // Example stuff:
    label: String,

    #[serde(skip)] // This how you opt-out of serialization of a field
    value: f32,

    #[serde(skip)]
    inf: Option<RenderCtx>,

    current_shader_text: String,
}

pub struct RenderCtx {
    dep_graph: CompleteGraphicsDependencyGraph,
    tex_id: TextureId,
}

impl Default for App {
    fn default() -> Self {
        Self {
            // Example stuff:
            label: "Hello World!".to_owned(),
            value: 2.7,
            inf: None,
            current_shader_text: shaderwheels_logic::rendering::DEFAULT_COMPUTE.to_string(),
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
        if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            Default::default()
        }
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

        if let None = self.inf {
            let renderstate = _frame.wgpu_render_state().unwrap();
            let draw_size = (512u32, 512u32);
            let texture = renderstate.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("drawtexture"),
                size: Extent3d {
                    width: draw_size.0,
                    height: draw_size.1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::COPY_DST
                    | wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            });
            let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
            let tex_id = renderstate.renderer.write().register_native_texture(
                &renderstate.device,
                &view,
                eframe::wgpu::FilterMode::Linear,
            );

            let rendergraph = CompleteGraphicsDependencyGraph::new(CompleteGraphicsInitialConfig {
                output_view: Some(view.clone()),
                output_format: Some(TextureFormat::Rgba8Unorm),
                hardware: Some(GPUAdapterInfo {
                    deviceref: renderstate.device.clone(),
                    queueref: renderstate.queue.clone(),
                }),
                shader_text: None,
                entry_point: None,
                preoutput_size: Some((512, 512)),
            });

            let mut rctx = RenderCtx {
                dep_graph: rendergraph,
                tex_id,
            };

            rctx.dep_graph
                .set_shader_text(shaderwheels_logic::rendering::DEFAULT_COMPUTE.to_string());
            rctx.dep_graph.set_entry_point("main".to_string());

            self.inf = Some(rctx);
        }

        let mut popup = false;

        ctx.input_mut(|i| {
            if i.consume_shortcut(&KeyboardShortcut::new(Modifiers::CTRL, egui::Key::S)) {
                popup = true;
            }
        });

        if popup {
            self.label = "BOB".to_string();
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

            ui.with_layout(Layout::bottom_up(egui::Align::Min), |bottom_ui| {
                if let Some(rctx) = self.inf.as_mut() {
                    let rt = if let Some(err) = rctx.dep_graph.get_compilation_error() {
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
                        RichText::new("Compilation successful.")
                    };

                    bottom_ui.label(rt.size(14f32));
                }
            });

            if changed {
                if let Some(rctx) = self.inf.as_mut() {
                    rctx.dep_graph
                        .set_shader_text(self.current_shader_text.clone());
                }
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let rect = ui.max_rect();
            if let Some(rctx) = self.inf.as_mut() {
                let uv = Rect {
                    min: pos2(0.0f32, 0.0f32),
                    max: pos2(1.0f32, 1.0f32),
                };
                rctx.dep_graph.mark_for_rerender();
                let success = pollster::block_on(rctx.dep_graph.complete());
                if success {
                    ui.painter().image(rctx.tex_id, rect, uv, Color32::WHITE);
                }
            } else {
                ui.painter().rect(
                    rect,
                    1.0f32,
                    Color32::WHITE,
                    Stroke::new(2.0f32, Color32::BLACK),
                    egui::StrokeKind::Inside,
                );
            }
        });
    }
}
