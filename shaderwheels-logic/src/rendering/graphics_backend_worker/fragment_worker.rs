use cardigan_incremental::{GeneralVersionedComp, Versioned, VersionedInputs, memoized};
use wgpu::wgt::BufferDescriptor;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, CommandEncoderDescriptor, ComputePassDescriptor, ComputePipeline,
    ComputePipelineDescriptor, PipelineLayout, ShaderModule, ShaderStages, Surface, TextureView,
    util::TextureBlitter,
};
use wgpu::{
    BlendState, Buffer, BufferBinding, BufferUsages, Color, ColorTargetState, FragmentState,
    MultisampleState, Operations, PrimitiveState, RenderPassColorAttachment, RenderPassDescriptor,
    RenderPipeline, RenderPipelineDescriptor, ShaderModuleDescriptor, TextureFormat, VertexState,
};

use crate::rendering::{DEFAULT_WGSL_VERT, WGSL_VERT_ENTRY};
use crate::rendering::graphics_backend_worker::shared::{
    BackendWorker, pipeline_layout, preoutput_texture_view,
};
use crate::rendering::shader_config::GPUAdapterInfo;

#[memoized]
async fn pipeline(
    hardware: &GPUAdapterInfo,
    pipeline_layout: &PipelineLayout,
    vert_module: &ShaderModule,
    frag_module: &ShaderModule,
    output_format: TextureFormat,
    vert_entry_point: &String,
    frag_entry_point: &String,
) -> Result<RenderPipeline, wgpu::Error> {
    // https://sotrh.github.io/learn-wgpu/beginner/tutorial3-pipeline/#how-do-we-use-the-shaders
    let comp_opts = wgpu::PipelineCompilationOptions::default();

    hardware
        .deviceref
        .push_error_scope(wgpu::ErrorFilter::Validation);

    let descriptor = RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(pipeline_layout),
        cache: None,
        vertex: VertexState {
            module: vert_module,
            entry_point: Some(&vert_entry_point),
            compilation_options: comp_opts.clone(),
            buffers: &[],
        },
        primitive: PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: None,
            unclipped_depth: false,
            polygon_mode: wgpu::PolygonMode::Fill,
            conservative: false,
        },
        depth_stencil: None,
        multisample: MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        fragment: Some(FragmentState {
            module: frag_module,
            entry_point: Some(&frag_entry_point),
            compilation_options: comp_opts,
            targets: &[Some(ColorTargetState {
                format: output_format,
                blend: Some(BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        multiview: None,
    };

    let pipeline = hardware.deviceref.create_render_pipeline(&descriptor);

    let errs = hardware.deviceref.pop_error_scope().await;
    if let Some(e) = errs {
        log::info!("{:?}", e);
        return Err(e);
    }
    Ok(pipeline)
}

#[memoized]
async fn bind_group_layout(hardware: &GPUAdapterInfo) -> BindGroupLayout {
    let layout = hardware
        .deviceref
        .create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("fragment bind group layout"),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
    layout
}

#[memoized]
async fn unif_buffer(hardware: &GPUAdapterInfo) -> Buffer {
    let buf = hardware.deviceref.create_buffer(&BufferDescriptor {
        label: Some("Position Uniform Buffer"),
        size: 32,
        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    buf
}

#[memoized]
async fn bind_group(
    hardware: &GPUAdapterInfo,
    bgl: &BindGroupLayout,
    unif_buffer: &Buffer,
) -> BindGroup {
    let bg = hardware.deviceref.create_bind_group(&BindGroupDescriptor {
        label: Some("Bind group!"),
        layout: bgl,
        entries: &[BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::Buffer(BufferBinding {
                buffer: unif_buffer,
                offset: 0,
                size: None,
            }),
        }],
    });
    bg
}

#[memoized]
async fn populate_uniforms(
    hardware: &GPUAdapterInfo,
    bf: &Buffer,
    preout_view_size: (u32, u32),
) -> () {

    let var_name = [preout_view_size.0, preout_view_size.1];
    let bytes = bytemuck::bytes_of(&var_name);
    println!("{:?} {:?} {:?} {:?} ; {:?} {:?} {:?} {:?}", bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7]);
    hardware.queueref.write_buffer(
        bf,
        0,
        bytemuck::bytes_of(&[preout_view_size.0, preout_view_size.1]),
    );
}

async fn recompute_preout_fn(
    pipeline: Option<&RenderPipeline>,
    preout_view: Option<&TextureView>,
    bind_group: Option<&BindGroup>,
    encoder: &mut wgpu::CommandEncoder,
) -> Option<()> {
    let preout_view = preout_view?;
    let bind_group = bind_group?;
    let pipeline = pipeline?;

    let render_pass_descriptor = RenderPassDescriptor {
        label: Some("Render Pass"),
        color_attachments: &[Some(RenderPassColorAttachment {
            view: preout_view,
            resolve_target: None,
            ops: Operations {
                load: wgpu::LoadOp::Clear(Color::BLUE),
                store: wgpu::StoreOp::Store,
            },
        })],
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
    };

    let mut render_pass = encoder.begin_render_pass(&render_pass_descriptor);
    render_pass.set_pipeline(pipeline);

    // TODO: Bind more groups
    render_pass.set_bind_group(0, bind_group, &[]);

    render_pass.draw(0..3, 0..1);

    Some(())
}

async fn render_output(
    hardware: Option<&GPUAdapterInfo>,
    bg: Option<&BindGroup>,
    pipeline: Option<&RenderPipeline>,
    blitter: Option<&TextureBlitter>,
    output_view: Option<&TextureView>,
    preout_view_size: &Option<(u32, u32)>,
    preout_view: Option<&TextureView>,
    uniform_values: Option<()>,
    recompute_preout: bool,
    rerender_out: bool,
) -> Option<()> {
    let hardware = hardware.as_ref()?;
    let blitter = blitter.as_ref()?;
    let output_view = output_view.as_ref()?;
    let preout_view = preout_view?;
    let _ = uniform_values?;

    let encoder_descriptor = CommandEncoderDescriptor {
        label: Some("Command Encoder Descriptor"),
    };
    let mut encoder = hardware
        .deviceref
        .create_command_encoder(&encoder_descriptor);

    if recompute_preout {
        recompute_preout_fn(pipeline, Some(preout_view), bg, &mut encoder).await;
    }

    if rerender_out {
        blitter.copy(&hardware.deviceref, &mut encoder, preout_view, output_view);
    }

    hardware.queueref.submit([encoder.finish()]);

    Some(())
}

#[derive(Default)]
pub struct FragmentWorkerPart {
    vert: Versioned<ShaderModule>,
    pov: preoutput_texture_view,
    pll: pipeline_layout,
    pl: pipeline,
    bgl: bind_group_layout,
    bf: unif_buffer,
    uv: populate_uniforms,
    bg: bind_group,
    preout_comp: GeneralVersionedComp<5>,
    rendered_comp: VersionedInputs<2>,
}

impl BackendWorker for FragmentWorkerPart {
    async fn step(
        &mut self,
        preout_size: &Versioned<(u32, u32)>,
        hardware: &Versioned<&GPUAdapterInfo>,
        module: &Versioned<&ShaderModule>,
        entry_point: &Versioned<&String>,
        blitter: &Versioned<&TextureBlitter>,
        render_output_on_invalidated: bool,
        output_view: &Option<&TextureView>,
    ) -> bool {
        let uses = wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_DST
            | wgpu::TextureUsages::RENDER_ATTACHMENT;
        let uses_vwrapped = Versioned::default();
        let uses_vwrapped = uses_vwrapped.next(Some(uses));
        let vert_ep = Versioned::default().next(Some(WGSL_VERT_ENTRY.to_string()));

        if let Some(hw) = hardware.get_value() {
            if self.vert.get_value().is_none() {
                self.vert = Versioned::default().next(Some(hw.deviceref.create_shader_module(
                    ShaderModuleDescriptor {
                        label: Some("vertex module"),
                        source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Owned(
                            DEFAULT_WGSL_VERT.to_string(),
                        )),
                    },
                )));
            }
        }

        let out_fmt = Versioned::default();
        let out_fmt = out_fmt.next(Some(TextureFormat::Rgba8Unorm));

        let bindgroup_lay = self.bgl.compute(hardware).await.my_as_ref();
        let pipeline_lay = self.pll.compute(hardware, &bindgroup_lay).await.my_as_ref();
        let preout_view = self
            .pov
            .compute(preout_size, &uses_vwrapped, hardware)
            .await
            .my_as_ref();

        let bf = self.bf.compute(hardware).await.my_as_ref();

        let bindgroup = self
            .bg
            .compute(hardware, &bindgroup_lay, &bf)
            .await
            .my_as_ref();

        let pipeline = self
            .pl
            .compute(
                hardware,
                &pipeline_lay,
                &self.vert.my_as_ref(),
                module,
                &out_fmt,
                &vert_ep.my_as_ref(),
                entry_point,
            )
            .await
            .my_as_ref();

        let safe_pipeline = pipeline.map(|f| match f {
            Some(Ok(p)) => Some(p),
        _ => None,
        });

        let unif_vals = self.uv.compute(hardware, &bf, preout_size).await;

        if render_output_on_invalidated && output_view.is_some() {
            let recompute_preout = self.preout_comp.check_and_update(&[
                *preout_size.version(),
                *pipeline.version(),
                *bindgroup.version(),
                *blitter.version(),
                *unif_vals.version(),
            ]);

            let rerender_out = self
                .rendered_comp
                .check_and_update(&[self.preout_comp.get_version(), *blitter.version()]);

            if rerender_out || recompute_preout {
                let res = render_output(
                    *hardware.get_value(),
                    *bindgroup.get_value(),
                    *safe_pipeline.get_value(),
                    *blitter.get_value(),
                    *output_view,
                    preout_size.get_value(),
                    *preout_view.get_value(),
                    *unif_vals.get_value(),
                    recompute_preout,
                    rerender_out,
                )
                .await;

                match res {
                    Some(_) => return true,
                    None => return false,
                }
            }
        }
        return false;
    }
}
