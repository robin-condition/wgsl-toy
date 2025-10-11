use eframe::egui_wgpu::RenderState;
use egui::{pos2, Color32, Rect, TextureId, Ui};
use shaderwheels_logic::rendering::{
    graphics_backend_client::GraphicsClient,
    shader_config::{GPUAdapterInfo, ShaderConfig},
};
use wgpu::{Device, Extent3d, TextureDescriptor, TextureFormat, TextureView};

use crate::app::egui_shaderwheels_logic;

pub struct TextureInfo {
    pub view: TextureView,
    pub id: TextureId,
    pub size: (u32, u32),
}

pub struct RenderCtx {
    pub client: GraphicsClient,
    pub present_buffer: Option<TextureInfo>,
    pub backend_buffer: Option<TextureInfo>,
}

impl Default for RenderCtx {
    fn default() -> Self {
        Self {
            client: GraphicsClient::new(ShaderConfig::default()),
            present_buffer: Default::default(),
            backend_buffer: Default::default(),
        }
    }
}

pub(crate) fn onetime_hardware_setup(cc: &eframe::CreationContext<'_>) -> RenderCtx {
    let renderstate = cc.wgpu_render_state.as_ref().unwrap();
    let draw_size = (512u32, 512u32);

    let mut client = GraphicsClient::new(ShaderConfig::default());
    client.set_hardware(GPUAdapterInfo {
        deviceref: renderstate.device.clone(),
        queueref: renderstate.queue.clone(),
    });

    let targ_size = (512, 512);

    client.set_preout_size(targ_size);
    /*CompleteGraphicsDependencyGraph::new(CompleteGraphicsInitialConfig {
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
    */
    let mut rctx = RenderCtx {
        client,
        present_buffer: None,
        backend_buffer: None,
    };

    rctx.present_buffer = Some(create_texture_info(renderstate, targ_size));
    rctx.backend_buffer = Some(create_texture_info(renderstate, targ_size));
    rctx.client
        .set_output_view(rctx.backend_buffer.as_ref().unwrap().view.clone());
    rctx
}

fn fix_texture_info(
    egui_renderstate: &RenderState,
    correct_size: (u32, u32),
    to_fix: &mut Option<TextureInfo>,
) {
    if let Some(val) = to_fix {
        if val.size == correct_size {
            return;
        }

        egui_renderstate.renderer.write().free_texture(&val.id);
    }

    *to_fix = Some(create_texture_info(egui_renderstate, correct_size));
}

pub(crate) fn prep_to_render(
    egui_renderstate: &RenderState,
    rctx: &mut RenderCtx,
    correct_size: (u32, u32),
) -> Option<TextureId> {
    if rctx.client.get_should_swap() {
        println!("Render succeeded, swap!");
        swap_backend_buffer_and_update_backbuffer_and_inform_client(
            egui_renderstate,
            rctx,
            correct_size,
        );
    }
    if rctx.client.get_preout_size() != Some(correct_size) {
        rctx.client.set_preout_size(correct_size);
    }

    rctx.present_buffer.as_ref().map(|f| f.id)
}

fn swap_backend_buffer_and_update_backbuffer_and_inform_client(
    egui_renderstate: &RenderState,
    rctx: &mut RenderCtx,
    correct_size: (u32, u32),
) {
    std::mem::swap(&mut rctx.backend_buffer, &mut rctx.present_buffer);
    fix_texture_info(egui_renderstate, correct_size, &mut rctx.backend_buffer);
    rctx.client
        .set_output_view(rctx.backend_buffer.as_ref().unwrap().view.clone());
}

fn create_texture_info(egui_renderstate: &RenderState, size: (u32, u32)) -> TextureInfo {
    let view = create_texture(&egui_renderstate.device, size);
    let id = egui_renderstate.renderer.write().register_native_texture(
        &egui_renderstate.device,
        &view,
        eframe::wgpu::FilterMode::Linear,
    );
    TextureInfo { view, id, size }
}

fn create_texture(dev: &Device, size: (u32, u32)) -> TextureView {
    let texture = dev.create_texture(&TextureDescriptor {
        label: Some("OUTPUT TEXTURE"),
        size: Extent3d {
            width: size.0,
            height: size.1,
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
    view
}

pub(crate) fn draw(rctx: &mut RenderCtx, renderstate: &RenderState, ui: &mut Ui) {
    let rect = ui.available_rect_before_wrap();
    let cur_size = (rect.width() as u32, rect.height() as u32);

    /*
    let retexture = rctx
        .client
        .get_preout_size()
        .map_or(true, |f| f != cur_size);


    if retexture {
        egui_shaderwheels_logic::replace_base_texture(&renderstate, rctx, cur_size);
    }*/

    let id_to_render = prep_to_render(renderstate, rctx, cur_size);
    //println!("Testing for render");

    if let Some(tex_id) = id_to_render {
        //println!("Retrieved tex id");
        let uv = Rect {
            min: pos2(0.0f32, 0.0f32),
            max: pos2(1.0f32, 1.0f32),
        };
        //rctx.dep_graph.mark_for_rerender();

        let success = true; //rctx.dep_graph.complete();

        if success {
            ui.painter().image(tex_id, rect, uv, Color32::WHITE);
        }
    }
}
