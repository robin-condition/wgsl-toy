use leptos::{html::Canvas, prelude::*, svg::text, task::spawn_local};
use reactive_stores::Store;
use wgpu::{util::TextureBlitterBuilder, BackendOptions, Backends, SurfaceConfiguration, SurfaceTarget};
//use wgpu::*;


#[derive(Debug, Default, Store)]
struct GlobalState {/*
    surface: Option<wgpu::Surface<'static>>,
    device: Option<wgpu::Device>,
    queue: Option<wgpu::Queue>*/
}



#[component]
fn App() -> impl IntoView {
    let (count, set_count) = signal(0);

    let node_ref = NodeRef::<Canvas>::new();
    provide_context(Store::new(GlobalState::default()));

    Effect::new(move |_| {
        if let Some(node) = node_ref.get() {
            let ctx = expect_context::<Store<GlobalState>>();
            // https://github.com/gfx-rs/wgpu/blob/trunk/examples/standalone/02_hello_window/src/main.rs
            let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor{
                //backends: Backends::GL,
                //flags: todo!(),
                //memory_budget_thresholds: todo!(),
                //backend_options: BackendOptions::,
                ..Default::default()
            });
            spawn_local(async move {
                let width = node.width();
                println!("{width}");
                let height = node.height();
            
                let surf_targ = SurfaceTarget::Canvas(node);
                let surface = instance.create_surface(surf_targ).unwrap();

                let adapter = instance
                .request_adapter(&wgpu::RequestAdapterOptions {
                    compatible_surface: Some(&surface),
                    ..Default::default()
                }).await.unwrap();
            
                let (device, queue) = adapter
                    .request_device(&wgpu::DeviceDescriptor::default())
                    .await
                    .unwrap();
                
                
                let cap = surface.get_capabilities(&adapter);
        let surface_format = cap.formats[0];

                let surface_config = SurfaceConfiguration{
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
                
                let text_view = {
                    // https://sotrh.github.io/learn-wgpu/beginner/tutorial5-textures/#loading-an-image-from-a-file

                    let bytes = include_bytes!("wgsl-hi.png");
                    let img = image::load_from_memory(bytes).unwrap();
                    let rgba = img.to_rgba8();
                    
                    let dimensions = rgba.dimensions();

                    let tex_size = wgpu::Extent3d {
                        width: dimensions.0,
                        height: dimensions.1,

                        depth_or_array_layers: 1
                    };

                    let texture = device.create_texture(&wgpu::TextureDescriptor {
                        size: tex_size,
                        mip_level_count: 1,
                        sample_count: 1,
                        dimension: wgpu::TextureDimension::D2,
                        format: wgpu::TextureFormat::Rgba8UnormSrgb,
                        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                        label: Some("hi_tex"),
                        view_formats: &[]
                    });

                    queue.write_texture(wgpu::TexelCopyTextureInfo {
                        texture: &texture,
                        mip_level: 0,
                        origin: wgpu::Origin3d::ZERO,
                        aspect: wgpu::TextureAspect::All,

                    }, &rgba,
                    wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(4 * dimensions.0),
                        rows_per_image: Some(dimensions.1)
                    },
                    tex_size
                    );

                    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
                    view
                };

                // render

                let surface_texture = 
            surface
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

        // Renders a GREEN screen
        let mut encoder = device.create_command_encoder(&Default::default());
        // Create the renderpass which will clear the screen.
        let renderpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &texture_view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });


        // If you wanted to call any drawing commands, they would go here.

        // End the renderpass.
        drop(renderpass);

        TextureBlitterBuilder::new(&device, surface_format.add_srgb_suffix())
        .sample_type(wgpu::FilterMode::Linear)
        .build()
        .copy(&device, &mut encoder, &text_view, &texture_view);

        // Submit the command in the queue to execute
        queue.submit([encoder.finish()]);
        //window.pre_present_notify();
        surface_texture.present();
            }
        );
        }
    });

    view! {
        <button
            on:click=move |_| set_count.set(count.get() + 1)
        >
            "Click me: "
            {count}
        </button>
        <p>
            "Double count: "
            {move || count.get() * 2}
        </p>
        <canvas width=500 height=500 node_ref=node_ref>
        </canvas>
    }
}

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(|| App())
}
