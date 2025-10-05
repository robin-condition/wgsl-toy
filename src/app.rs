use eframe::wgpu::Texture;
use egui::{
    pos2, Color32, ImageSource, KeyboardShortcut, Modifiers, Rect, Stroke, TextureHandle,
    TextureId, Vec2,
};
use shaderwheels_logic::rendering::{
    CompleteGraphicsDependencyGraph, CompleteGraphicsInitialConfig, GPUAdapterInfo,
};
use wgpu::{Extent3d, TextureFormat, TextureView};

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct App {
    // Example stuff:
    label: String,

    #[serde(skip)] // This how you opt-out of serialization of a field
    value: f32,

    #[serde(skip)]
    inf: Option<RenderCtx>,
}

pub struct RenderCtx {
    dep_graph: CompleteGraphicsDependencyGraph,
    out_tex: TextureView,
    tex_id: TextureId,
}

impl Default for App {
    fn default() -> Self {
        Self {
            // Example stuff:
            label: "Hello World!".to_owned(),
            value: 2.7,
            inf: None,
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
                out_tex: view,
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

        egui::TopBottomPanel::bottom("editor_panel").show(ctx, |ui| {});

        egui::CentralPanel::default().show(ctx, |ui| {
            let rect = ui.max_rect();
            if let Some(rctx) = self.inf.as_mut() {
                let uv = Rect {
                    min: pos2(0.0f32, 0.0f32),
                    max: pos2(1.0f32, 1.0f32),
                };
                rctx.dep_graph.mark_for_rerender();
                let success = futures::executor::block_on(rctx.dep_graph.complete());
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

fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Powered by ");
        ui.hyperlink_to("egui", "https://github.com/emilk/egui");
        ui.label(" and ");
        ui.hyperlink_to(
            "eframe",
            "https://github.com/emilk/egui/tree/master/crates/eframe",
        );
        ui.label(".");
    });
}
