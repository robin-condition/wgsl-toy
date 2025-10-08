use cardigan_incremental::{GeneralVersionedComp, Versioned, VersionedInputs, memoized};
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, CommandEncoderDescriptor, ComputePassDescriptor, ComputePipeline,
    ComputePipelineDescriptor, PipelineLayout, ShaderModule, ShaderStages, Surface, TextureView,
    util::TextureBlitter,
};

use crate::rendering::graphics_backend_worker::shared::{
    BackendWorker, GPUAdapterInfo, pipeline_layout, preoutput_texture_view,
};

#[memoized]
async fn pipeline(
    hardware: &GPUAdapterInfo,
    pipeline_layout: &PipelineLayout,
    module: &ShaderModule,
    entry_point: &String,
) -> ComputePipeline {
    let comp_opts = wgpu::PipelineCompilationOptions::default();

    let descriptor = ComputePipelineDescriptor {
        label: Some("Compute Pipeline"),
        layout: Some(pipeline_layout),
        module: module,
        entry_point: Some(entry_point.as_ref()),
        compilation_options: comp_opts,
        cache: None,
    };

    let pipeline = hardware.deviceref.create_compute_pipeline(&descriptor);
    pipeline
}

#[memoized]
async fn bind_group_layout(hardware: &GPUAdapterInfo) -> BindGroupLayout {
    let layout = hardware
        .deviceref
        .create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("bind group layout"),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::COMPUTE,
                ty: wgpu::BindingType::StorageTexture {
                    access: wgpu::StorageTextureAccess::WriteOnly,
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    view_dimension: wgpu::TextureViewDimension::D2,
                },
                count: None,
            }],
        });
    layout
}

#[memoized]
async fn bind_group(
    hardware: &GPUAdapterInfo,
    bgl: &BindGroupLayout,
    preout_view: &TextureView,
) -> BindGroup {
    let bg = hardware.deviceref.create_bind_group(&BindGroupDescriptor {
        label: Some("Bind group!"),
        layout: bgl,
        entries: &[BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::TextureView(preout_view),
        }],
    });
    bg
}

async fn recompute_preout_fn(
    pipeline: Option<&ComputePipeline>,
    preout_view_size: &Option<(u32, u32)>,
    bind_group: Option<&BindGroup>,
    encoder: &mut wgpu::CommandEncoder,
) -> Option<()> {
    let preout_view_size = preout_view_size.as_ref()?;
    let bind_group = bind_group?;
    let pipeline = pipeline?;

    let compute_pass_descriptor = ComputePassDescriptor {
        label: Some("Compute Pass!"),
        timestamp_writes: None,
    };

    let mut compute_pass = encoder.begin_compute_pass(&compute_pass_descriptor);
    compute_pass.set_pipeline(pipeline);

    // TODO: Bind more groups
    compute_pass.set_bind_group(0, bind_group, &[]);

    let workgroup_counts = (
        preout_view_size.0.div_ceil(16u32),
        preout_view_size.1.div_ceil(16u32),
    );

    compute_pass.dispatch_workgroups(workgroup_counts.0, workgroup_counts.1, 1);

    Some(())
}

async fn render_output(
    hardware: Option<&GPUAdapterInfo>,
    bg: Option<&BindGroup>,
    pipeline: Option<&ComputePipeline>,
    blitter: Option<&TextureBlitter>,
    output_view: Option<&TextureView>,
    preout_view_size: &Option<(u32, u32)>,
    preout_view: Option<&TextureView>,
    recompute_preout: bool,
    rerender_out: bool,
) -> Option<()> {
    let hardware = hardware.as_ref()?;
    let blitter = blitter.as_ref()?;
    let output_view = output_view.as_ref()?;
    let preout_view = preout_view.as_ref()?;

    let encoder_descriptor = CommandEncoderDescriptor {
        label: Some("Command Encoder Descriptor"),
    };
    let mut encoder = hardware
        .deviceref
        .create_command_encoder(&encoder_descriptor);

    if recompute_preout {
        recompute_preout_fn(pipeline, preout_view_size, bg, &mut encoder).await;
    }

    if rerender_out {
        blitter.copy(&hardware.deviceref, &mut encoder, preout_view, output_view);
    }

    hardware.queueref.submit([encoder.finish()]);

    Some(())
}

pub struct GPUExactSurface<'a> {
    pub surface: Surface<'a>,
}

pub struct ComputeWorkerPart {
    pov: preoutput_texture_view,
    pll: pipeline_layout,
    pl: pipeline,
    bgl: bind_group_layout,
    bg: bind_group,
    preout_comp: GeneralVersionedComp<4>,
    rendered_comp: VersionedInputs<3>,
}

impl ComputeWorkerPart {
    async fn rerender(&mut self) -> bool {
        todo!()
    }
}

impl BackendWorker for ComputeWorkerPart {
    async fn step(
        &mut self,
        preout_size: &Versioned<(u32, u32)>,
        hardware: &Versioned<&GPUAdapterInfo>,
        module: &Versioned<&ShaderModule>,
        entry_point: &Versioned<&String>,
        blitter: &Versioned<&TextureBlitter>,
        render_output_on_invalidated: bool,
        output_view: &Versioned<&TextureView>,
    ) -> bool {
        let uses = wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_DST
            | wgpu::TextureUsages::STORAGE_BINDING;

        let uses_vwrapped = Versioned::default();
        let uses_vwrapped = uses_vwrapped.next(Some(uses));

        let bindgroup_lay = self.bgl.compute(hardware).await.my_as_ref();
        let pipeline_lay = self.pll.compute(hardware, &bindgroup_lay).await.my_as_ref();
        let preout_view = self
            .pov
            .compute(preout_size, &uses_vwrapped, hardware)
            .await
            .my_as_ref();

        let bindgroup = self
            .bg
            .compute(hardware, &bindgroup_lay, &preout_view)
            .await
            .my_as_ref();

        let pipeline = self
            .pl
            .compute(hardware, &pipeline_lay, module, entry_point)
            .await
            .my_as_ref();

        if render_output_on_invalidated {
            let recompute_preout = self.preout_comp.check_and_update(&[
                *preout_size.version(),
                *pipeline.version(),
                *bindgroup.version(),
                *blitter.version(),
            ]);

            let rerender_out = self.rendered_comp.check_and_update(&[
                self.preout_comp.get_version(),
                *blitter.version(),
                *output_view.version(),
            ]);

            if rerender_out || recompute_preout {
                let res = render_output(
                    *hardware.get_value(),
                    *bindgroup.get_value(),
                    *pipeline.get_value(),
                    *blitter.get_value(),
                    *output_view.get_value(),
                    preout_size.get_value(),
                    *preout_view.get_value(),
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
