
use std::borrow::Cow;

use wgpu::{
    BindGroup, ComputePipeline, Device, PipelineLayout, Queue, ShaderModuleDescriptor, Surface,
    SurfaceConfiguration, SurfaceTarget, TextureFormat, TextureView,
    util::{TextureBlitter, TextureBlitterBuilder},
};

pub struct GPUAdapterPrep<'a> {
    surface: Surface<'a>,
    device: Device,
    queue: Queue,
    surface_format: TextureFormat,
    texture_dimensions: (u32, u32),
    view: TextureView,
    bind_group: BindGroup,
    pipeline_layout: PipelineLayout,
    blitter: TextureBlitter,
}

pub const DEFAULT_COMPUTE: &str = include_str!("compute.wgsl");

pub struct PipelinePrep {
    pipeline: ComputePipeline,
}

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

    GPUAdapterPrep {
        surface,
        device,
        queue,
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
