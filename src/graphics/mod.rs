pub mod camera;
pub mod common_models; // TODO: probably don't reexport this
mod debug_pipeline;
mod shader;
mod texture;
pub mod textured_pipeline; // TODO: probably don't reexport this

use std::sync::Arc;

use anyhow::Context;
use cgmath::Vector2;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, BufferBindingType, BufferUsages, CommandEncoderDescriptor,
    LoadOp, PowerPreference, RenderPassColorAttachment, RenderPassDescriptor,
    RequestAdapterOptions, ShaderStages, StoreOp, Surface, SurfaceConfiguration, TextureUsages,
    TextureViewDescriptor,
    util::{BufferInitDescriptor, DeviceExt},
};
use winit::{dpi::LogicalSize, window::Window};

use crate::graphics::{
    camera::Camera2DUniform,
    debug_pipeline::DebugPipeline,
    textured_pipeline::{TexturedPipeline, TexturedQuad},
};

pub struct GraphicsState {
    window: Arc<Window>,
    surface: Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: SurfaceConfiguration,

    camera: Camera2DUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: BindGroup,

    textured_pipeline: TexturedPipeline,
    debug_pipeline: DebugPipeline,
}

impl GraphicsState {
    pub async fn new(window: Arc<Window>) -> anyhow::Result<Self> {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone()).unwrap();

        // Adapter corresponds to a physical graphics and/or compute device
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::default(),
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await?;

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                memory_hints: Default::default(),
                trace: wgpu::Trace::Off,
            })
            .await?;

        let surface_capabilities = surface.get_capabilities(&adapter);

        let surface_format = {
            let mut surface_format = None;
            for format in surface_capabilities.formats {
                if format.is_srgb() {
                    surface_format = Some(format);
                    break;
                }
            }
            surface_format.unwrap()
        };

        let window_size = window.inner_size();

        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: window_size.width,
            height: window_size.height,
            present_mode: surface_capabilities.present_modes[0],
            desired_maximum_frame_latency: 2,
            alpha_mode: surface_capabilities.alpha_modes[0],
            view_formats: vec![],
        };

        surface.configure(&device, &config);

        let window_size = window.inner_size();
        let scale_factor = window.scale_factor();
        let logical_size: LogicalSize<f32> = window_size.to_logical(scale_factor);

        let camera = Camera2DUniform::new(logical_size.width, logical_size.height);
        let camera_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Debug Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("Debug Camera Bind Group Layout"),
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let camera_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Debug Camera Bind Group"),
            layout: &camera_bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        let textured_pipeline =
            TexturedPipeline::new(&device, &queue, &camera_bind_group_layout, &config)
                .context("Failed to make textured pipeline")?;
        let debug_pipeline = DebugPipeline::new(&device, &config, &camera_bind_group_layout);

        Ok(Self {
            window,
            surface,
            device,
            queue,
            config,
            camera,
            camera_buffer,
            camera_bind_group,
            textured_pipeline,
            debug_pipeline,
        })
    }

    pub fn get_logical_size(&self) -> LogicalSize<f32> {
        let window_size = self.window.inner_size();
        let scale_factor = self.window.scale_factor();
        window_size.to_logical(scale_factor)
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        // TODO: is it possible to get zero size?
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
        // TODO: need to reacquire logical size of the window?
    }

    pub fn render(&mut self) -> anyhow::Result<()> {
        let output = self
            .surface
            .get_current_texture()
            .with_context(|| "Failed to get current texture on render")?;

        let view = output
            .texture
            .create_view(&TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Render pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            self.textured_pipeline.render(
                &mut self.queue,
                &mut render_pass,
                &self.camera_bind_group,
            );

            self.debug_pipeline
                .render(&mut render_pass, &self.camera_bind_group);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        Ok(())
    }

    pub fn push_textured_quad(&mut self, quad: TexturedQuad) {
        self.textured_pipeline.push_textured_quad(quad)
    }

    pub fn push_debug_square(
        &mut self,
        position: Vector2<f32>,
        scale: Vector2<f32>,
        rotation: f32,
        color: (f32, f32, f32),
    ) {
        self.debug_pipeline
            .push_square(&self.queue, position, scale, rotation, color);
    }

    pub fn push_debug_triangle(
        &mut self,
        position: Vector2<f32>,
        scale: Vector2<f32>,
        rotation: f32,
        color: (f32, f32, f32),
    ) {
        self.debug_pipeline
            .push_triangle(&self.queue, position, scale, rotation, color);
    }

    pub fn clear_instances(&mut self) {
        self.textured_pipeline.clear_instances();
        self.debug_pipeline.clear_instances();
    }
}
