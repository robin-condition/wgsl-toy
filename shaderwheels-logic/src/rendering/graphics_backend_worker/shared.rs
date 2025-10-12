use std::borrow::Cow;

use cardigan_incremental::{Versioned, memoized};
use wgpu::{
    BindGroupLayout, Device, Extent3d, PipelineLayout, PipelineLayoutDescriptor, Queue,
    ShaderModule, ShaderModuleDescriptor, Texture, TextureDescriptor, TextureFormat, TextureView,
    TextureViewDescriptor,
    util::{TextureBlitter, TextureBlitterBuilder},
};

use crate::rendering::shader_config::{GPUAdapterInfo, ShaderLanguage};

#[memoized]
async fn preoutput_texture_view(
    preout_size: (u32, u32),
    uses: wgpu::TextureUsages,
    hardware: &GPUAdapterInfo,
) -> TextureView {
    let descriptor = TextureDescriptor {
        label: Some("Compute Result"),
        size: Extent3d {
            width: preout_size.0,
            height: preout_size.1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,

        dimension: wgpu::TextureDimension::D2,
        format: TextureFormat::Rgba8Unorm,
        usage: uses,
        //wgpu::TextureUsages::TEXTURE_BINDING
        //  | wgpu::TextureUsages::COPY_DST
        // | wgpu::TextureUsages::STORAGE_BINDING,
        view_formats: &[],
    };

    let preout_tex = hardware.deviceref.create_texture(&descriptor);

    let view_descript = TextureViewDescriptor::default();
    let tex_view = preout_tex.create_view(&view_descript);

    tex_view
}

#[memoized]
async fn blitter(hardware: &GPUAdapterInfo, output_format: TextureFormat) -> TextureBlitter {
    TextureBlitterBuilder::new(&hardware.deviceref, output_format).build()
}

#[memoized]
async fn pipeline_layout(hardware: &GPUAdapterInfo, bgl: &BindGroupLayout) -> PipelineLayout {
    let pipeline_layout = hardware
        .deviceref
        .create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Pipeline layout!"),
            bind_group_layouts: &[bgl],
            push_constant_ranges: &[],
        });
    pipeline_layout
}

pub trait BackendWorker {
    async fn step(
        &mut self,
        preout_size: &Versioned<(u32, u32)>,
        hardware: &Versioned<&GPUAdapterInfo>,
        module: &Versioned<&ShaderModule>,
        entry_point: &Versioned<&String>,
        blitter: &Versioned<&TextureBlitter>,
        render_output_on_invalidated: bool,
        output_view: &Option<&TextureView>,
    ) -> bool;
}

#[memoized]
async fn module_comp(
    device: &Device,
    shader_text: &String,
    lang: ShaderLanguage,
) -> ModuleCompResult {
    device.push_error_scope(wgpu::ErrorFilter::Validation);

    let module = device.create_shader_module(ShaderModuleDescriptor {
        label: Some("Compute Module"),
        source: match lang {
            //ShaderLanguage::Glsl => wgpu::ShaderSource::Glsl {shader: (Cow::Owned(shader_text.clone())), stage: Sh, },
            ShaderLanguage::Wgsl => wgpu::ShaderSource::Wgsl(Cow::Owned(shader_text.clone())),
        },
    });

    //let res = module.get_compilation_info().await;
    //res.messages[0].location.unwrap()

    let errs = device.pop_error_scope().await;

    if let Some(err) = errs {
        log::info!("{:?}", err);
        Err(err)
    } else {
        Ok(module)
    }
}

pub type ModuleCompResult = Result<ShaderModule, wgpu::Error>;
