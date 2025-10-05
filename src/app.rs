use eframe::egui_wgpu::RenderState;
use egui::{
    pos2, Color32, KeyboardShortcut, Layout, Modifiers, Rect, RichText, Stroke, TextureId, Widget,
};
use egui_code_editor::ColorTheme;
use shaderwheels_logic::rendering::{
    CompleteGraphicsDependencyGraph, CompleteGraphicsInitialConfig, GPUAdapterInfo,
};
use wgpu::{wgt::TextureDescriptor, Extent3d, TextureFormat};

use crate::app::eguice_syntax::wgsl_syntax;

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

#[derive(Default)]
pub struct RenderCtx {
    dep_graph: CompleteGraphicsDependencyGraph,
    tex_id: Option<TextureId>,
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

        let mut rctx = App::onetime_hardware_setup(cc);

        rctx.dep_graph
            .set_shader_text(state.current_shader_text.clone());
        rctx.dep_graph.set_entry_point("main".to_string());

        Self { ..state }
    }

    fn onetime_hardware_setup(cc: &eframe::CreationContext<'_>) -> RenderCtx {
        let renderstate = cc.wgpu_render_state.as_ref().unwrap();
        let draw_size = (512u32, 512u32);

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
            recompute_on_invalidate: true,
        });

        let mut rctx = RenderCtx {
            dep_graph: rendergraph,
            tex_id: None,
        };

        App::replace_base_texture(renderstate, &mut rctx, draw_size);
        rctx
    }

    fn replace_base_texture(
        egui_renderstate: &RenderState,
        ctx: &mut RenderCtx,
        new_size: (u32, u32),
    ) {
        if let Some(id) = ctx.tex_id {
            egui_renderstate.renderer.write().free_texture(&id);
            ctx.tex_id = None;
        }

        let texture = egui_renderstate.device.create_texture(&TextureDescriptor {
            label: Some("OUTPUT TEXTURE"),
            size: Extent3d {
                width: new_size.0,
                height: new_size.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let tex_id = egui_renderstate.renderer.write().register_native_texture(
            &egui_renderstate.device,
            &view,
            eframe::wgpu::FilterMode::Linear,
        );

        ctx.dep_graph.set_preoutput_size(new_size);
        ctx.dep_graph.set_output_view(view);

        ctx.tex_id = Some(tex_id);
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
                        RichText::new("Latest compilation successful.")
                    };

                    bottom_ui.label(rt.size(14f32));
                }
            });

            if roi_changed {
                if let Some(rctx) = self.inf.as_mut() {
                    rctx.dep_graph.recompute_on_invalidation = self.recompute_on_invalidate;
                }
            }

            if (changed || rou_changed) && self.compile_on_change {
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
