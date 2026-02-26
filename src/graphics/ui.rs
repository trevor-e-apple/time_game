use wgpu::{RenderPass, SurfaceConfiguration};
use wgpu_text::{
    BrushBuilder, TextBrush,
    glyph_brush::{self, OwnedSection, ab_glyph::FontRef},
};

pub struct UI<'a> {
    brush: TextBrush<FontRef<'a>>,
    sections: Vec<OwnedSection>,
}

impl UI<'_> {
    pub fn new(device: &wgpu::Device, config: &SurfaceConfiguration) -> Self {
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
        Self {
            sections: vec![],
            brush,
        }
    }

    pub fn add_text(&mut self, text: &String, width: f32, height: f32, x: f32, y: f32) {
        let section = glyph_brush::Section::default()
            .add_text(glyph_brush::Text::new(text))
            .with_bounds((width, height))
            .with_layout(
                glyph_brush::Layout::default()
                    .v_align(glyph_brush::VerticalAlign::Center)
                    .line_breaker(glyph_brush::BuiltInLineBreaker::AnyCharLineBreaker),
            )
            .with_screen_position((x, y))
            .to_owned();
        self.sections.push(section);
    }

    pub fn render(
        &mut self,
        render_pass: &mut RenderPass,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) {
        self.brush.queue(device, queue, &self.sections).unwrap();
        self.brush.draw(render_pass);
    }
}
