use std::borrow::Cow;

use leptos::{html::Canvas, logging, prelude::*};
use web_sys::HtmlCanvasElement;
use wgpu::{
    ShaderModuleDescriptor, SurfaceConfiguration, SurfaceTarget, util::TextureBlitterBuilder,
};

struct GPUPrepState {}

async fn do_prep(node: HtmlCanvasElement) -> GPUPrepState {
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        //backends: Backends::GL,
        //flags: todo!(),
        //memory_budget_thresholds: todo!(),
        //backend_options: BackendOptions::,
        ..Default::default()
    });

    let width = node.width();

    let height = node.height();

    let surf_targ = SurfaceTarget::Canvas(node);
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

    let module = device.create_shader_module(ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(Cow::Owned("".to_owned())),
    });

    let surface_config = SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface_format,
        view_formats: vec![surface_format.add_srgb_suffix()],
        alpha_mode: wgpu::CompositeAlphaMode::Auto,
        width: width,
        height: height,
        desired_maximum_frame_latency: 2,
        present_mode: wgpu::PresentMode::AutoVsync,
    };

    surface.configure(&device, &surface_config);

    // https://sotrh.github.io/learn-wgpu/beginner/tutorial5-textures/#loading-an-image-from-a-file

    let bytes = include_bytes!("wgsl-hi.png");
    let img = image::load_from_memory(bytes).unwrap();
    let rgba = img.to_rgba8();

    let dimensions = rgba.dimensions();

    let tex_size = wgpu::Extent3d {
        width: dimensions.0,
        height: dimensions.1,

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

    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &rgba,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(4 * dimensions.0),
            rows_per_image: Some(dimensions.1),
        },
        tex_size,
    );

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

    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: None,
        layout: Some(&pipeline_layout),
        module: &module,
        entry_point: Some("main"),
        compilation_options: wgpu::PipelineCompilationOptions::default(),
        cache: None,
    });

    // render

    let surface_texture = surface
        .get_current_texture()
        .expect("failed to acquire next swapchain texture");
    let texture_view = surface_texture
        .texture
        .create_view(&wgpu::TextureViewDescriptor {
            // Without add_srgb_suffix() the image we will be working with
            // might not be "gamma correct".
            format: Some(surface_format.add_srgb_suffix()),
            ..Default::default()
        });

    let mut encoder = device.create_command_encoder(&Default::default());

    let mut computepass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
        label: Some("MyPass"),
        timestamp_writes: None,
    });

    computepass.set_pipeline(&pipeline);

    computepass.set_bind_group(0, &bind_group, &[]);

    let workgroup_size = (16, 16);

    let workgroup_counts = (
        dimensions.0.div_ceil(workgroup_size.0),
        dimensions.1.div_ceil(workgroup_size.1),
    );
    logging::log!("counts: {:?}", workgroup_counts);
    logging::log!("img size: {:?}", dimensions);
    computepass.dispatch_workgroups(workgroup_counts.0, workgroup_counts.1, 1);

    drop(computepass);

    TextureBlitterBuilder::new(&device, surface_format.add_srgb_suffix())
        .sample_type(wgpu::FilterMode::Linear)
        .build()
        .copy(&device, &mut encoder, &view, &texture_view);

    // Submit the command in the queue to execute
    queue.submit([encoder.finish()]);
    //window.pre_present_notify();

    surface_texture.present();

    GPUPrepState {}
}

#[component]
pub fn ComputeCanvas(
    #[prop(into)] size: Signal<(u32, u32)>,
    #[prop(into)] shader_text: Signal<String>,
) -> impl IntoView {
    let node_ref = NodeRef::<Canvas>::new();
    let canvas_exists = move || node_ref.get().is_some();

    let (prep_state, set_prep_state) = signal_local(None);
    let prep_done = move || prep_state.read().is_some();

    Effect::new(move |_| {
        if let Some(node) = node_ref.get() {
            if !prep_done() {
                set_prep_state.set(Some(do_prep(node)));
            }
            // https://github.com/gfx-rs/wgpu/blob/trunk/examples/standalone/02_hello_window/src/main.rs
        }
    });

    view! { <canvas width=move || size.get().0 height=move || size.get().1 node_ref=node_ref></canvas> }
}
