use std::borrow::Cow;

use wgpu::{
    BindGroup, BindGroupLayout, CommandEncoderDescriptor, ComputePassDescriptor, ComputePipeline,
    ComputePipelineDescriptor, Device, Extent3d, PipelineCompilationOptions, PipelineLayout, Queue,
    ShaderModule, ShaderModuleDescriptor, Surface, SurfaceConfiguration, SurfaceTarget, Texture,
    TextureDescriptor, TextureFormat, TextureView, TextureViewDescriptor,
    util::{TextureBlitter, TextureBlitterBuilder},
};

pub struct CompleteGraphicsDependencyGraph {
    // Inputs
    hardware: Option<GPUAdapterInfo>,
    unif_values: Option<()>,
    output_format: Option<OutputFormat>,
    preoutput_size: Option<(u32, u32)>,
    // This should really be some kind of renderoptions
    output_view: Option<OutputTextureView>,
    // Pretty dang critical
    shader_text: Option<String>,
    entry_point: Option<String>,

    // Computation results and scratchpad
    uniform_contents_correct: bool,
    adapter_prep: Option<GPUAdapterPrep>,
    preoutput_tex: Option<PreoutputTexture>,
    preoutput_tex_view: Option<TextureView>,
    module: Option<ShaderModule>,
    pipeline: Option<ComputePipeline>,
    preout_texture_rendered: bool,
    output_view_rendered: bool,
}

impl CompleteGraphicsDependencyGraph {

    pub fn new() -> Self {
        todo!()
    }

    // Any of the setters:
    // 1. Update their corresponding input field.
    // 2. Invalidate direct usages.
    // Indirect usages are invalidated by the updater.
    // The updater proceeds in topological order,
    // so usages will be invalidated before they are run.

    pub fn set_shader_text(&mut self, text: String) {
        self.shader_text = Some(text);
        
        // Invalidate module.
        self.module = None;
    }

    pub fn set_entry_point(&mut self, text: String) {
        self.entry_point = Some(text);
        
        // Invalidate pipeline.
        self.pipeline = None;
    }

    pub fn set_uniform_contents(&mut self, contents: ()) {
        
        // Invalidate uniform values on gpu
        self.uniform_contents_correct = false;

        // TODO: Update some kind of uniform value configuration
        todo!();
    }

    pub fn set_output_view(&mut self, output_view: TextureView) {
        self.output_view = Some(OutputTextureView { output_view });

        // Invalidate the render / mark for rerender.
        self.output_view_rendered = false;
    }

    pub fn set_output_format(&mut self, output_format: TextureFormat) {
        self.output_format = Some(OutputFormat { format: output_format });

        // TODO: invalidate the blitter.
        todo!()
    }

    pub fn set_preoutput_size(&mut self, preout_size: (u32, u32)) {
        self.preoutput_size = Some(preout_size);

        // Invalidate preout texture.
        self.preoutput_tex = None;
    }

    pub fn mark_for_rerender(&mut self) {
        // All this does is invalidate.
        self.output_view_rendered = false;
    }

    // Recomputes all necessary or invalidated steps.
    pub async fn complete(&mut self) {
        // Create the compute result ("preout") texture
        if let None = self.preoutput_tex {
            // Invalidate preout view
            self.preoutput_tex_view = None;
            // Invalidate preout render
            self.preout_texture_rendered = false;

            self.try_make_preout_tex();
        }

        // Create the module from the shader text
        if let None = self.module {
            // Invalidate pipeline
            self.pipeline = None;

            self.try_make_module();
        }

        // Create the compute pipeline
        if let None = self.pipeline {
            // Invalidate compute result
            self.preout_texture_rendered = false;

            self.try_make_pipeline();
        }

        // TODO: Create bind groups from specification

        // TODO: Set uniform values from inputs

        // Create preoutput texture view
        if let None = self.preoutput_tex_view {
            // Invalidate compute result
            self.preout_texture_rendered = false;
            // Invalidate copied draw? Gonna skip for now
            //self.output_view_rendered = false;

            self.try_make_preout_view();
        }

        // Any GPU work to do at all, make encoder for it.
        if !self.output_view_rendered
        // Actually.. let's only do GPU work if something is actually to be rendered. So I'll comment this out
        //|| !self.preout_texture_rendered
        {
            // Nothing to invalidate -- we're the whole ball game.
            // Even the compute output does not invalidate the drawn version, because that has to be externally requested.

            self.try_render_output();
        }
    }

    fn try_render_output(&mut self) -> Option<()> {
        let hardware = self.hardware.as_ref()?;

        let bind_group_info = self.adapter_prep.as_ref()?;

        // These are `?`'d now even though they aren't needed until later because if we don't have them, we can't copy to render,
        // so we should just skip for now.
        let output_view = self.output_view.as_ref()?;
        let preout_tex_view = self.preoutput_tex_view.as_ref()?;

        let encoder_descriptor = CommandEncoderDescriptor {
            label: Some("Command Encoder Descriptor"),
        };
        let mut encoder = hardware
            .deviceref
            .create_command_encoder(&encoder_descriptor);

        // Rerun compute shader, render compute result ("preout") texture
        // Or, try to. IF it has been invalidated.
        if !self.preout_texture_rendered {
            // Nothing to invalidate

            // This is NOT `?`'d because I want to continue to copy to render screen even if this is a fail.
            let pipeline_maybe = self.pipeline.as_ref();
            let preout_size_maybe = self.preoutput_size;

            match (pipeline_maybe, preout_size_maybe) {
                (Some(pipeline), Some(preout_size)) => {
                    CompleteGraphicsDependencyGraph::try_recompute(
                        pipeline,
                        preout_size,
                        bind_group_info,
                        &mut encoder,
                    );

                    // Mark this render as done only if that's true
                    self.preout_texture_rendered = true;
                }
                _ => (),
            }
        }

        // Blit/copy the compute result to the output view
        if !self.output_view_rendered {
            bind_group_info.blitter.copy(
                &hardware.deviceref,
                &mut encoder,
                preout_tex_view,
                &output_view.output_view,
            );

            self.output_view_rendered = true;
        }

        hardware.queueref.submit([encoder.finish()]);

        Some(())
    }

    fn try_recompute(
        pipeline: &ComputePipeline,
        preout_size: (u32, u32),
        bind_group_info: &GPUAdapterPrep,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        let compute_pass_descriptor = ComputePassDescriptor {
            label: Some("Compute Pass!"),
            timestamp_writes: None,
        };

        let mut compute_pass = encoder.begin_compute_pass(&compute_pass_descriptor);
        compute_pass.set_pipeline(pipeline);

        // TODO: Bind more groups
        compute_pass.set_bind_group(0, &bind_group_info.bind_group, &[]);

        let workgroup_counts = (preout_size.0.div_ceil(16u32), preout_size.1.div_ceil(16u32));

        compute_pass.dispatch_workgroups(workgroup_counts.0, workgroup_counts.1, 1);
    }

    fn try_make_preout_view(&mut self) -> Option<()> {
        let tex = self.preoutput_tex.as_ref()?;

        let view_descript = TextureViewDescriptor::default();
        let tex_view = tex.texture.create_view(&view_descript);

        self.preoutput_tex_view = Some(tex_view);
        Some(())
    }

    fn try_make_pipeline(&mut self) -> Option<()> {
        let hardware = self.hardware.as_ref()?;
        let layouts = self.adapter_prep.as_ref()?;

        let module = self.module.as_ref()?;
        let entry_point = self.entry_point.as_ref()?;

        let comp_opts = wgpu::PipelineCompilationOptions::default();

        let descriptor = ComputePipelineDescriptor {
            label: Some("Compute Pipeline"),
            layout: Some(&layouts.pipeline_layout),
            module: module,
            entry_point: Some(entry_point.as_ref()),
            compilation_options: comp_opts,
            cache: None,
        };

        let pipeline = hardware.deviceref.create_compute_pipeline(&descriptor);
        self.pipeline = Some(pipeline);
        Some(())
    }

    fn try_make_module(&mut self) -> Option<()> {
        let hardware = self.hardware.as_ref()?;
        let shader_text = self.shader_text?;
        let module = hardware
            .deviceref
            .create_shader_module(ShaderModuleDescriptor {
                label: Some("Compute Module"),
                source: Cow::Owned(shader_text),
            });
        self.module = Some(module);
        Some(())
    }

    fn try_make_preout_tex(&mut self) -> Option<()> {
        let preout_size = self.preoutput_size?;

        let hardware = self.hardware.as_ref()?;

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
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::STORAGE_BINDING,
            view_formats: &[],
        };

        let preout_tex = PreoutputTexture {
            texture: hardware.deviceref.create_texture(&descriptor),
            size: preout_size,
        };
        self.preoutput_tex = Some(preout_tex);

        Some(())
    }
}

pub struct GPUAdapterInfo {
    pub deviceref: Device,
    pub queueref: Queue,
}

pub struct GPUExactSurface<'a> {
    pub surface: Surface<'a>,
}

pub struct OutputFormat {
    pub format: TextureFormat,
}

pub struct OutputTextureView {
    pub output_view: TextureView,
}

pub struct PreoutputTexture {
    pub texture: Texture,
    pub size: (u32, u32),
}

pub struct GPUAdapterPrep {
    bind_group: BindGroup,
    pipeline_layout: PipelineLayout,
    blitter: TextureBlitter,
}

pub const DEFAULT_COMPUTE: &str = include_str!("compute.wgsl");

pub struct PipelinePrep {
    pipeline: ComputePipeline,
}

pub async fn create_device_info_no_surface() {}

pub async fn prep_wgpu<'window>(
    surf_targ: SurfaceTarget<'window>,
    surface_size: (u32, u32),
) -> GPUAdapterPrep<'window> {
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        //backends: Backends::GL,
        //flags: todo!(),
        //memory_budget_thresholds: todo!(),
        //backend_options: BackendOptions::,
        ..Default::default()
    });

    let texture_size = surface_size;

    let surface = instance.create_surface(surf_targ).unwrap();

    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            compatible_surface: Some(&surface),
            ..Default::default()
        })
        .await
        .unwrap();

    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor::default())
        .await
        .unwrap();

    let cap = surface.get_capabilities(&adapter);
    let surface_format = cap.formats[0];

    let surface_config = SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface_format,
        view_formats: vec![surface_format.add_srgb_suffix()],
        alpha_mode: wgpu::CompositeAlphaMode::Auto,
        width: surface_size.0,
        height: surface_size.1,
        desired_maximum_frame_latency: 2,
        present_mode: wgpu::PresentMode::AutoVsync,
    };

    surface.configure(&device, &surface_config);

    // https://sotrh.github.io/learn-wgpu/beginner/tutorial5-textures/#loading-an-image-from-a-file

    let tex_size = wgpu::Extent3d {
        width: texture_size.0,
        height: texture_size.1,

        depth_or_array_layers: 1,
    };

    let texture = device.create_texture(&wgpu::TextureDescriptor {
        size: tex_size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_DST
            | wgpu::TextureUsages::STORAGE_BINDING,
        label: Some("hi_tex"),
        view_formats: &[],
    });

    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

    // https://github.com/gfx-rs/wgpu/blob/trunk/examples/standalone/01_hello_compute/src/main.rs
    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Compute Inputs"),
        entries: &[
            // https://www.reddit.com/r/wgpu/comments/x5z4tb/comment/in42y6p/?utm_source=share&utm_medium=web3x&utm_name=web3xcss&utm_term=1&utm_content=share_button
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::StorageTexture {
                    access: wgpu::StorageTextureAccess::WriteOnly,
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    view_dimension: wgpu::TextureViewDimension::D2,
                },
                count: None,
            },
        ],
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Bind group"),
        layout: &bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::TextureView(&view),
        }],
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("pipelin elaouyt"),
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
    });

    // render, queue: wgpu::Queue

    let blitter = TextureBlitterBuilder::new(&device, surface_format.add_srgb_suffix())
        .sample_type(wgpu::FilterMode::Linear)
        .build();

    let adapter_stuff = GPUAdapterInfo {
        deviceref: device,
        queueref: queue,
    };

    GPUAdapterPrep {
        surface,
        surface_format,
        texture_dimensions: texture_size,
        view,
        bind_group,
        pipeline_layout,
        blitter,
    }
}

pub fn prep_shader(prep: &GPUAdapterPrep, shader_text: String) -> PipelinePrep {
    let module = prep.device.create_shader_module(ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(Cow::Owned(shader_text)),
    });

    let pipeline = prep
        .device
        .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: Some(&prep.pipeline_layout),
            module: &module,
            entry_point: Some("main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });

    PipelinePrep { pipeline }
}

pub fn do_shader(gpu_prep: &GPUAdapterPrep, shader_prep: &PipelinePrep) {
    let surface_texture = gpu_prep
        .surface
        .get_current_texture()
        .expect("failed to acquire next swapchain texture");
    let texture_view = surface_texture
        .texture
        .create_view(&wgpu::TextureViewDescriptor {
            // Without add_srgb_suffix() the image we will be working with
            // might not be "gamma correct".
            format: Some(gpu_prep.surface_format.add_srgb_suffix()),
            ..Default::default()
        });

    let mut encoder = gpu_prep.device.create_command_encoder(&Default::default());

    let mut computepass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
        label: Some("MyPass"),
        timestamp_writes: None,
    });

    computepass.set_pipeline(&shader_prep.pipeline);

    computepass.set_bind_group(0, &gpu_prep.bind_group, &[]);

    let workgroup_size = (16, 16);

    let workgroup_counts = (
        gpu_prep.texture_dimensions.0.div_ceil(workgroup_size.0),
        gpu_prep.texture_dimensions.1.div_ceil(workgroup_size.1),
    );
    //logging::log!("counts: {:?}", workgroup_counts);
    //logging::log!("img size: {:?}", gpu_prep.texture_dimensions);
    computepass.dispatch_workgroups(workgroup_counts.0, workgroup_counts.1, 1);

    drop(computepass);

    gpu_prep.blitter.copy(
        &gpu_prep.device,
        &mut encoder,
        &gpu_prep.view,
        &texture_view,
    );

    // Submit the command in the queue to execute
    gpu_prep.queue.submit([encoder.finish()]);
    //window.pre_present_notify();

    surface_texture.present();
}
