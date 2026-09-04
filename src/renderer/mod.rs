// renderer/mod.rs
use super::Gresult;
use std::sync::Arc;
use wgpu::wgt::{CommandEncoderDescriptor, DeviceDescriptor, TextureViewDescriptor};
use wgpu::*;
use winit::dpi::PhysicalSize;
use winit::window::Window;

mod utils;
#[allow(unused)]
use utils::*;
#[allow(unused)]
pub struct Gpu {
    window: Arc<Window>,
    surface: Surface<'static>,
    adapter: Adapter,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    pipeline: RenderPipeline,
}

impl Gpu {
    pub async fn init(window: Arc<Window>) -> Gresult<Self> {
        let instance = Instance::new(InstanceDescriptor::new_without_display_handle());

        let surface = instance.create_surface(window.clone())?;

        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
                apply_limit_buckets: false,
            })
            .await;

        let adapter = match adapter {
            Ok(a) => a,
            Err(_) => {
                instance
                    .request_adapter(&RequestAdapterOptions {
                        power_preference: PowerPreference::LowPower,
                        force_fallback_adapter: true,
                        compatible_surface: Some(&surface),
                        apply_limit_buckets: false,
                    })
                    .await?
            }
        };

        let (device, queue) = adapter.request_device(&DeviceDescriptor::default()).await?;

        let width = window.inner_size().width;
        let height = window.inner_size().height;
        let config = surface.get_default_config(&adapter, width, height);

        let config = config.ok_or_else(
            || return "Failed graphics initialisation. No Compatible Surface was found",
        )?;

        surface.configure(&device, &config);

        let shader = device.create_shader_module(include_wgsl!("./shaders/triangle.wgsl"));
        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("RenderPipeline"),
            layout: None,
            vertex: VertexState {
                module: &shader,
                entry_point: None,
                compilation_options: PipelineCompilationOptions::default(),
                buffers: &[],
            },
            primitive: PrimitiveState::default(),
            depth_stencil: None,
            multisample: MultisampleState::default(),
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: None,
                compilation_options: PipelineCompilationOptions::default(),
                targets: &[Some(ColorTargetState {
                    format: config.format,
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });

        Ok(Self {
            window,
            surface,
            adapter,
            device,
            queue,
            config,
            pipeline,
        })
    }

    pub fn resize(&mut self, s: PhysicalSize<u32>) /*-> Gresult<()>*/ {
        self.config.width = s.width;
        self.config.height = s.height;

        if self.config.width == 0 || self.config.height == 0 {return}; 

        self.surface.configure(&self.device, &self.config);
        //Ok(())
    }

    pub fn render(&mut self, _focus: bool) -> Gresult<()> {
        let output = self.surface.get_current_texture();

        let tex = match output {
            CurrentSurfaceTexture::Success(t) => t,
            CurrentSurfaceTexture::Suboptimal(_) => {println!("suboptimal"); return Ok(())},
            _ => return Ok(())
        };

        let mut encoder = self.device.create_command_encoder(&CommandEncoderDescriptor::default());

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[
                    // This is what @location(0) in the fragment shader targets
                    Some(wgpu::RenderPassColorAttachment {
                        view: &tex.texture.create_view(&TextureViewDescriptor::default()),
                        resolve_target: None,
                        depth_slice: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 1.0,
                                g: 1.0,
                                b: 1.0,
                                a: 1.0,
                            }),
                            store: wgpu::StoreOp::Store,
                        },
                    }),
                ],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });

            render_pass.set_pipeline(&self.pipeline);
            render_pass.draw(0..3, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        self.queue.present(tex);
        Ok(())
    }
}
