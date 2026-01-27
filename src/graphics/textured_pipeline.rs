use std::mem;

use anyhow::Context;
use cgmath::{Matrix3, Vector2};
use image::GenericImageView;
use wgpu::{
    AddressMode, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, BlendState,
    BufferDescriptor, BufferUsages, ColorTargetState, ColorWrites, Extent3d, Face, FilterMode,
    FragmentState, FrontFace, IndexFormat, MultisampleState, Origin3d, PipelineCompilationOptions,
    PipelineLayoutDescriptor, PolygonMode, PrimitiveState, PrimitiveTopology, RenderPass,
    RenderPipeline, RenderPipelineDescriptor, SamplerBindingType, ShaderStages,
    SurfaceConfiguration, TexelCopyBufferLayout, TexelCopyTextureInfo, TextureAspect,
    TextureDimension, TextureFormat, TextureSampleType, TextureUsages, TextureViewDescriptor,
    TextureViewDimension, VertexAttribute, VertexBufferLayout, VertexFormat, VertexState,
    VertexStepMode,
    util::{BufferInitDescriptor, DeviceExt},
    wgt::{SamplerDescriptor, TextureDescriptor},
};

use crate::graphics::{common_models::SQUARE_INDICES, shader::load_shader};

const MAX_TRIANGLES: usize = 128;
const MAX_QUADS: usize = 1024;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex2 {
    pub position: [f32; 2],
    pub tex_coords: [f32; 2],
}

impl Vertex2 {
    pub fn buffer_layout() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex2>() as wgpu::BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &[
                VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: VertexFormat::Float32x2,
                },
                VertexAttribute {
                    offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: VertexFormat::Float32x2,
                },
            ],
        }
    }
}

pub const TRIANGLE_VERTICES: &[Vertex2] = &[
    Vertex2 {
        position: [0.0, 0.5],
        tex_coords: [0.0, 0.0], // Debug code, not currently set
    },
    Vertex2 {
        position: [-0.5, -0.5],
        tex_coords: [0.0, 0.0],
    },
    Vertex2 {
        position: [0.5, -0.5],
        tex_coords: [0.0, 0.0],
    },
];

pub const SQUARE_VERTICES: &[Vertex2] = &[
    Vertex2 {
        position: [-0.5, 0.5],
        tex_coords: [0.0, 0.0],
    },
    Vertex2 {
        position: [0.5, -0.5],
        tex_coords: [1.0, 1.0],
    },
    Vertex2 {
        position: [0.5, 0.5],
        tex_coords: [1.0, 0.0],
    },
    Vertex2 {
        position: [-0.5, -0.5],
        tex_coords: [0.0, 1.0],
    },
];

pub struct TexturedInstance {
    pub position: Vector2<f32>,
    pub scale: Vector2<f32>,
    pub rotation: cgmath::Rad<f32>,
}

// TODO: does this need to be public?
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct InstanceRaw {
    model: [[f32; 3]; 3],
}

impl InstanceRaw {
    fn buffer_layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 6]>() as wgpu::BufferAddress,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

impl TexturedInstance {
    fn to_raw(&self) -> InstanceRaw {
        InstanceRaw {
            model: (Matrix3::from_translation(self.position)
                * Matrix3::from_angle_z(self.rotation)
                * Matrix3::from_nonuniform_scale(self.scale.x, self.scale.y))
            .into(),
        }
    }
}

struct Model {
    vertex_buffer: wgpu::Buffer,
    num_vertices: u32,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
    instance_buffer: wgpu::Buffer,
    num_instances: u32,
    max_instances: usize,
}

#[derive(Copy, Clone)]
pub struct TexturedQuad {
    pub position: Vector2<f32>,
    pub dimensions: Vector2<f32>,
    pub layer: u32,
    // TODO: we need a texture handle
}

pub struct TexturedPipeline {
    render_pipeline: RenderPipeline,
    models: Vec<Model>,
    diffuse_bind_group: BindGroup,
    quad_index: usize,
    textured_quads: Vec<TexturedQuad>,
}

impl TexturedPipeline {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        camera_bind_group_layout: &BindGroupLayout,
        config: &SurfaceConfiguration,
    ) -> anyhow::Result<Self> {
        // TODO: textures should come from a load function just like shaders do
        let (texture_bind_group_layout, diffuse_bind_group) = {
            let diffuse_bytes = include_bytes!("../../data/happy-tree.png");
            let diffuse_image =
                image::load_from_memory(diffuse_bytes).context("Failed to load texture")?;
            let diffuse_rgba = diffuse_image.to_rgba8();
            let dimensions = diffuse_image.dimensions();
            let texture_size = Extent3d {
                width: dimensions.0,
                height: dimensions.1,
                depth_or_array_layers: 1,
            };
            let diffuse_texture = device.create_texture(&TextureDescriptor {
                label: Some("Diffuse Texture"),
                size: texture_size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Rgba8UnormSrgb,
                usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
                view_formats: &[],
            });

            queue.write_texture(
                TexelCopyTextureInfo {
                    texture: &diffuse_texture,
                    mip_level: 0,
                    origin: Origin3d::ZERO,
                    aspect: TextureAspect::All,
                },
                &diffuse_rgba,
                TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * dimensions.0),
                    rows_per_image: Some(dimensions.1),
                },
                texture_size,
            );

            let diffuse_texture_view =
                diffuse_texture.create_view(&TextureViewDescriptor::default());
            let diffuse_sampler = device.create_sampler(&SamplerDescriptor {
                label: Some("Diffuse Sampler"),
                address_mode_u: AddressMode::ClampToEdge,
                address_mode_v: AddressMode::ClampToEdge,
                address_mode_w: AddressMode::ClampToEdge,
                mag_filter: FilterMode::Linear,
                min_filter: FilterMode::Nearest,
                mipmap_filter: FilterMode::Nearest,
                ..Default::default()
            });

            let texture_bind_group_layout =
                device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                    label: Some("Texture Bind Group Layout"),
                    entries: &[
                        BindGroupLayoutEntry {
                            binding: 0,
                            visibility: ShaderStages::FRAGMENT,
                            ty: BindingType::Texture {
                                multisampled: false,
                                view_dimension: TextureViewDimension::D2,
                                sample_type: TextureSampleType::Float { filterable: true },
                            },
                            count: None,
                        },
                        BindGroupLayoutEntry {
                            binding: 1,
                            visibility: ShaderStages::FRAGMENT,
                            ty: BindingType::Sampler(SamplerBindingType::Filtering),
                            count: None,
                        },
                    ],
                });
            let diffuse_bind_group = device.create_bind_group(&BindGroupDescriptor {
                label: Some("Diffuse Bind Group"),
                layout: &texture_bind_group_layout,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: BindingResource::TextureView(&diffuse_texture_view),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: BindingResource::Sampler(&diffuse_sampler),
                    },
                ],
            });

            (texture_bind_group_layout, diffuse_bind_group)
        };

        let render_pipeline = {
            let shader = load_shader(&device, "shader.wgsl", "Render pipeline shader");

            let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&texture_bind_group_layout, &camera_bind_group_layout],
                push_constant_ranges: &[],
            });

            let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&render_pipeline_layout),
                vertex: VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    compilation_options: PipelineCompilationOptions::default(),
                    buffers: &[Vertex2::buffer_layout(), InstanceRaw::buffer_layout()],
                },
                fragment: Some(FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    compilation_options: PipelineCompilationOptions::default(),
                    targets: &[Some(ColorTargetState {
                        format: config.format,
                        blend: Some(BlendState::REPLACE),
                        write_mask: ColorWrites::ALL,
                    })],
                }),
                primitive: PrimitiveState {
                    topology: PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: FrontFace::Ccw,
                    cull_mode: Some(Face::Back),
                    unclipped_depth: false,
                    polygon_mode: PolygonMode::Fill,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
                cache: None,
            });

            render_pipeline
        };

        let mut models = vec![];

        let quad_index = Self::add_model(
            &mut models,
            device,
            SQUARE_VERTICES,
            SQUARE_INDICES,
            MAX_QUADS,
        );

        Ok(Self {
            render_pipeline,
            models,
            diffuse_bind_group,
            quad_index,
            textured_quads: vec![],
        })
    }

    pub fn render(
        &mut self,
        queue: &wgpu::Queue,
        render_pass: &mut RenderPass<'_>,
        camera_bind_group: &BindGroup,
    ) {
        // Write quads to instance buffers
        {
            // Sort the quads by their layers
            self.textured_quads.sort_by_key(|k| k.layer);

            // Write quads to instance buffers
            for quad in &self.textured_quads {
                Self::add_instance(
                    &mut self.models,
                    queue,
                    self.quad_index,
                    TexturedInstance {
                        position: quad.position,
                        scale: quad.dimensions,
                        rotation: cgmath::Rad(0.0),
                    },
                );
            }
        }

        // Buffers are now set. Make render calls
        {
            render_pass.set_pipeline(&self.render_pipeline);
            // TODO: move this bind group set into the loop?
            render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
            render_pass.set_bind_group(1, camera_bind_group, &[]);

            for model in &self.models {
                render_pass.set_vertex_buffer(0, model.vertex_buffer.slice(..));
                render_pass.set_index_buffer(model.index_buffer.slice(..), IndexFormat::Uint32);
                render_pass.set_vertex_buffer(1, model.instance_buffer.slice(..));
                render_pass.draw_indexed(0..model.num_indices, 0, 0..model.num_instances);
            }
        }
    }

    fn add_model(
        models: &mut Vec<Model>,
        device: &wgpu::Device,
        vertices: &[Vertex2],
        indices: &[u32],
        max_instances: usize,
    ) -> usize {
        // TODO: Have a way to provide labels
        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(vertices),
            usage: BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(indices),
            usage: BufferUsages::INDEX,
        });
        let instance_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Instance Buffer"),
            size: (mem::size_of::<InstanceRaw>() * max_instances) as wgpu::BufferAddress,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let model_index = models.len();
        models.push(Model {
            vertex_buffer,
            num_vertices: vertices.len() as u32,
            index_buffer,
            num_indices: indices.len() as u32,
            instance_buffer,
            num_instances: 0,
            max_instances,
        });

        model_index
    }

    // TODO: maybe reallocate instance buffer if we exceed max instances?
    fn add_instance(
        models: &mut Vec<Model>,
        queue: &wgpu::Queue,
        model_index: usize,
        instance: TexturedInstance,
    ) {
        if let Some(model) = models.get_mut(model_index) {
            assert!(
                (model.num_instances as usize) < model.max_instances,
                "Exceeded maximum number of instances for model"
            );

            queue.write_buffer(
                &model.instance_buffer,
                (model.num_instances as usize * mem::size_of::<InstanceRaw>())
                    as wgpu::BufferAddress,
                bytemuck::cast_slice(&[instance.to_raw()]),
            );
            model.num_instances += 1;
        }
    }

    /// Clears push buffers in preparation for next frame update
    pub fn clear(&mut self) {
        self.textured_quads.clear();
        self.models[self.quad_index].num_instances = 0;
    }

    pub fn push_textured_quad(&mut self, quad: TexturedQuad) {
        self.textured_quads.push(quad);
    }
}
