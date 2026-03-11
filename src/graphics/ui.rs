use cgmath::Vector2;
use image::math::Rect;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, BufferBindingType, BufferUsages, RenderPass, ShaderStages,
    SurfaceConfiguration,
    util::{BufferInitDescriptor, DeviceExt},
};
use wgpu_text::{
    BrushBuilder, TextBrush,
    glyph_brush::{self, OwnedSection, ab_glyph::FontRef},
};
use winit::dpi::LogicalSize;

use crate::graphics::{camera::Camera2DUniform, no_textures_pipeline::NoTexturesPipeline};

pub struct UI<'a> {
    brush: TextBrush<FontRef<'a>>,
    sections: Vec<OwnedSection>,
    primitive_pipeline: NoTexturesPipeline,
    camera_bind_group: BindGroup, // Top left is origin
}

pub struct SectionHandle {
    index: usize,
}

impl UI<'_> {
    pub fn new(
        device: &wgpu::Device,
        config: &SurfaceConfiguration,
        logical_size: LogicalSize<f32>,
    ) -> Self {
        let brush = {
            let font = include_bytes!("../../data/DejaVuSans.ttf");
            let brush = Some(BrushBuilder::using_font_bytes(font).unwrap().build(
                &device,
                config.width,
                config.height,
                config.format,
            ))
            .unwrap();

            brush
        };

        let camera = Camera2DUniform::new_top_left_origin(logical_size.width, logical_size.height);
        let camera_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Debug Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("UI Camera Bind Group Layout"),
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
            label: Some("UI Camera Bind Group"),
            layout: &camera_bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });
        // Note that because we use the top_left_origin projection matrix, we actually end up
        // flipping our geometry. So we want the front face to be culled instead of the back.
        // All the geometry in this pipeline is untextured, so we don't need to invert the
        // texel y coordinates.
        let primitive_pipeline = NoTexturesPipeline::new(
            device,
            config,
            &camera_bind_group_layout,
            Some(wgpu::Face::Front),
        );

        Self {
            sections: vec![],
            brush,
            primitive_pipeline,
            camera_bind_group,
        }
    }

    pub fn push_text(
        &mut self,
        text: &String,
        font_size: f32,
        width: f32,
        height: f32,
        x: f32,
        y: f32,
        color: (f32, f32, f32),
    ) -> SectionHandle {
        let section = glyph_brush::Section::default()
            .add_text(
                glyph_brush::Text::new(text)
                    .with_color((color.0, color.1, color.2, 1.0))
                    .with_scale(font_size),
            )
            .with_bounds((width, height))
            .with_layout(
                glyph_brush::Layout::default_single_line().v_align(glyph_brush::VerticalAlign::Top),
            )
            .with_screen_position((x, y))
            .to_owned();

        let result = self.sections.len();
        self.sections.push(section);

        SectionHandle { index: result }
    }

    pub fn get_bounding_box(
        &mut self,
        section_handle: SectionHandle,
    ) -> (Vector2<f32>, Vector2<f32>) {
        let section = self.sections[section_handle.index].to_borrowed();

        match self.brush.glyph_bounds(section) {
            Some(rect) => {
                let top_left = Vector2 {
                    x: rect.min.x,
                    y: rect.min.y,
                };
                let dim = Vector2 {
                    x: rect.width(),
                    y: rect.height(),
                };
                (top_left, dim)
            }
            None => {
                // If glyph_bounds returns None, then the section is empty
                (Vector2 { x: 0.0, y: 0.0 }, Vector2 { x: 0.0, y: 0.0 })
            }
        }
    }

    pub fn push_square(
        &mut self,
        queue: &wgpu::Queue,
        position: Vector2<f32>,
        scale: Vector2<f32>,
        rotation: f32,
        color: (f32, f32, f32),
    ) {
        self.primitive_pipeline
            .push_square(queue, position, scale, rotation, color)
    }

    pub fn render(
        &mut self,
        render_pass: &mut RenderPass,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) {
        // TODO: z-buffering to allow intermixing
        self.primitive_pipeline
            .render(render_pass, &self.camera_bind_group);
        self.brush.queue(device, queue, &self.sections).unwrap();
        self.brush.draw(render_pass);
        self.sections.clear();
    }
}
