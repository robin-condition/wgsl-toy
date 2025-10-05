use eframe::egui_wgpu::RenderState;
use egui::{pos2, Color32, Rect, TextureId, Ui};
use shaderwheels_logic::rendering::{
    CompleteGraphicsDependencyGraph, CompleteGraphicsInitialConfig, GPUAdapterInfo,
};
use wgpu::{Extent3d, TextureDescriptor, TextureFormat};

use crate::app::egui_shaderwheels_logic;

#[derive(Default)]
pub struct RenderCtx {
    pub dep_graph: CompleteGraphicsDependencyGraph,
    pub tex_id: Option<TextureId>,
}

pub(crate) fn onetime_hardware_setup(cc: &eframe::CreationContext<'_>) -> RenderCtx {
    let renderstate = cc.wgpu_render_state.as_ref().unwrap();
    let draw_size = (512u32, 512u32);

    let rendergraph = CompleteGraphicsDependencyGraph::new(CompleteGraphicsInitialConfig {
        output_view: None,
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

    replace_base_texture(renderstate, &mut rctx, draw_size);
    rctx
}

pub(crate) fn replace_base_texture(
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

pub(crate) fn draw(rctx: &mut RenderCtx, renderstate: &RenderState, ui: &mut Ui) {
    let rect = ui.available_rect_before_wrap();
    let cur_size = (rect.width() as u32, rect.height() as u32);

    let retexture = rctx
        .dep_graph
        .get_preout_size()
        .map_or(true, |f| f != cur_size);

    if retexture {
        egui_shaderwheels_logic::replace_base_texture(&renderstate, rctx, cur_size);
    }

    if let Some(tex_id) = rctx.tex_id.as_ref() {
        let uv = Rect {
            min: pos2(0.0f32, 0.0f32),
            max: pos2(1.0f32, 1.0f32),
        };
        rctx.dep_graph.mark_for_rerender();
        let success = pollster::block_on(rctx.dep_graph.complete());
        if success {
            ui.painter().image(*tex_id, rect, uv, Color32::WHITE);
        }
    }
}
