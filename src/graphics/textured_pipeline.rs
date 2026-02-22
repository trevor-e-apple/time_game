use std::{collections::HashMap, env, fs::File, io::Read, mem, path::Path};

use anyhow::Context;
use cgmath::{Matrix3, Vector2};
use image::GenericImageView;
use wgpu::{
    AddressMode, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, BlendState,
    BufferDescriptor, BufferUsages, ColorTargetState, ColorWrites, Extent3d, Face, FilterMode,
    FragmentState, FrontFace, IndexFormat, MipmapFilterMode, MultisampleState, Origin3d,
    PipelineCompilationOptions, PipelineLayoutDescriptor, PolygonMode, PrimitiveState,
    PrimitiveTopology, RenderPass, RenderPipeline, RenderPipelineDescriptor, SamplerBindingType,
    ShaderStages, SurfaceConfiguration, TexelCopyBufferLayout, TexelCopyTextureInfo, TextureAspect,
    TextureDimension, TextureFormat, TextureSampleType, TextureUsages, TextureViewDescriptor,
    TextureViewDimension, VertexAttribute, VertexBufferLayout, VertexFormat, VertexState,
    VertexStepMode,
    util::{BufferInitDescriptor, DeviceExt},
    wgt::{SamplerDescriptor, TextureDescriptor},
};

use crate::graphics::mipmapper::Mipmapper;
use crate::graphics::{common_models::SQUARE_INDICES, shader::load_shader};

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

struct Quads {
    vertex_buffer: wgpu::Buffer,
    num_vertices: u32,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
    instance_buffer: wgpu::Buffer,
    num_instances: u32,
    max_instances: usize,
}

#[derive(Copy, Clone)]
struct TexturedQuad {
    position: Vector2<f32>,
    dimensions: Vector2<f32>,
    layer: u32, // NOTE: layers will be sorted from smallest to largest
    texture_handle: TextureHandle,
}

pub struct TexturedPipeline {
    render_pipeline: RenderPipeline,
    quads: Quads,
    texture_bind_group_layout: BindGroupLayout,
    textured_quads: Vec<TexturedQuad>,
    texture_manager: TextureManager,
}

impl TexturedPipeline {
    pub fn new(
        device: &wgpu::Device,
        camera_bind_group_layout: &BindGroupLayout,
        config: &SurfaceConfiguration,
    ) -> anyhow::Result<Self> {
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

        let texture_manager = TextureManager::new(device);

        let render_pipeline = {
            let shader = load_shader(&device, "shader.wgsl", "Render pipeline shader");

            let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&texture_bind_group_layout, &camera_bind_group_layout],
                immediate_size: 0,
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
                cache: None,
                multiview_mask: None,
            });

            render_pipeline
        };

        let quads = {
            // TODO: Have a way to provide labels
            let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(SQUARE_VERTICES),
                usage: BufferUsages::VERTEX,
            });
            let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(SQUARE_INDICES),
                usage: BufferUsages::INDEX,
            });
            let instance_buffer = device.create_buffer(&BufferDescriptor {
                label: Some("Instance Buffer"),
                size: (mem::size_of::<InstanceRaw>() * MAX_QUADS) as wgpu::BufferAddress,
                usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });

            Quads {
                vertex_buffer,
                num_vertices: SQUARE_VERTICES.len() as u32,
                index_buffer,
                num_indices: SQUARE_INDICES.len() as u32,
                instance_buffer,
                num_instances: 0,
                max_instances: MAX_QUADS,
            }
        };

        Ok(Self {
            render_pipeline,
            quads,
            texture_manager,
            texture_bind_group_layout,
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

            // Write quads to instance buffer
            // TODO: draw collections of quads on the same layer with the same texture as a single draw instruction
            for quad in &self.textured_quads {
                let instance = TexturedInstance {
                    position: quad.position,
                    scale: quad.dimensions,
                    rotation: cgmath::Rad(0.0),
                };
                queue.write_buffer(
                    &self.quads.instance_buffer,
                    (self.quads.num_instances as usize * mem::size_of::<InstanceRaw>())
                        as wgpu::BufferAddress,
                    bytemuck::cast_slice(&[instance.to_raw()]),
                );
                self.quads.num_instances += 1;
            }
        }

        // Buffers are now set. Make render calls
        {
            render_pass.set_pipeline(&self.render_pipeline);
            // TODO: move this bind group set into the loop?
            render_pass.set_bind_group(1, camera_bind_group, &[]);

            for (index, textured_quad) in self.textured_quads.iter().enumerate() {
                let bind_group = self
                    .texture_manager
                    .get_bind_group(&textured_quad.texture_handle)
                    .unwrap();
                render_pass.set_bind_group(0, bind_group, &[]);

                render_pass.set_vertex_buffer(0, self.quads.vertex_buffer.slice(..));
                render_pass
                    .set_index_buffer(self.quads.index_buffer.slice(..), IndexFormat::Uint32);
                render_pass.set_vertex_buffer(1, self.quads.instance_buffer.slice(..));
                render_pass.draw_indexed(
                    0..self.quads.num_indices,
                    0,
                    (index as u32)..((index + 1) as u32),
                );
            }
        }

        // Clear instances
        self.textured_quads.clear();
        self.quads.num_instances = 0;
    }

    pub fn push_textured_quad(
        &mut self,
        position: Vector2<f32>,
        dimensions: Vector2<f32>,
        layer: u32,
        texture_file_name: &str,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) {
        let texture_handle = self
            .texture_manager
            .load_texture(
                texture_file_name,
                &self.texture_bind_group_layout,
                device,
                queue,
            )
            .unwrap();
        let textured_quad = TexturedQuad {
            position,
            dimensions,
            layer,
            texture_handle,
        };
        self.textured_quads.push(textured_quad);
    }
}

struct TextureManager {
    loaded_textures: HashMap<String, usize>,
    bind_groups: Vec<BindGroup>,
    mipmapper: Mipmapper,
}

#[derive(Copy, Clone)]
struct TextureHandle {
    index: usize,
}

impl TextureManager {
    fn new(device: &wgpu::Device) -> Self {
        Self {
            mipmapper: Mipmapper::new(device),
            loaded_textures: HashMap::new(),
            bind_groups: vec![],
        }
    }

    fn load_texture(
        &mut self,
        texture_file_name: &str,
        texture_bind_group_layout: &BindGroupLayout,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> anyhow::Result<TextureHandle> {
        // TODO: handle case where we have too many textures in memory
        // Check if the texture is already loaded
        match self.loaded_textures.get(texture_file_name) {
            Some(texture_index) => Ok(TextureHandle {
                index: *texture_index,
            }),
            None => {
                let data_dir = env::var("DATA_DIR").unwrap();
                let texture_path = Path::new(&data_dir).join(texture_file_name);
                let mut texture_file = File::open(texture_path).unwrap();

                let mut buffer =
                    Vec::<u8>::with_capacity(texture_file.metadata().unwrap().len() as usize);
                texture_file.read_to_end(&mut buffer).unwrap();

                let image = image::load_from_memory(&buffer).context("Failed to load texture")?;
                let diffuse_rgba = image.to_rgba8();
                let dimensions = image.dimensions();
                let texture_size = Extent3d {
                    width: dimensions.0,
                    height: dimensions.1,
                    depth_or_array_layers: 1,
                };
                let mip_level_count = texture_size.width.min(texture_size.height).ilog2();
                let texture = device.create_texture(&TextureDescriptor {
                    label: Some(&format!("{} Texture", texture_file_name)),
                    size: texture_size,
                    mip_level_count: mip_level_count,
                    sample_count: 1,
                    dimension: TextureDimension::D2,
                    format: TextureFormat::Rgba8UnormSrgb,
                    usage: TextureUsages::TEXTURE_BINDING
                        | TextureUsages::COPY_SRC
                        | TextureUsages::COPY_DST,
                    view_formats: &[],
                });

                queue.write_texture(
                    TexelCopyTextureInfo {
                        texture: &texture,
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

                let texture_view = texture.create_view(&TextureViewDescriptor::default());
                let sampler = device.create_sampler(&SamplerDescriptor {
                    label: Some(&format!("{} Sampler", texture_file_name)),
                    address_mode_u: AddressMode::ClampToEdge,
                    address_mode_v: AddressMode::ClampToEdge,
                    address_mode_w: AddressMode::ClampToEdge,
                    mag_filter: FilterMode::Linear,
                    min_filter: FilterMode::Nearest,
                    mipmap_filter: MipmapFilterMode::Nearest,
                    ..Default::default()
                });

                let bind_group = device.create_bind_group(&BindGroupDescriptor {
                    label: Some(&format!("{} Bind Group", texture_file_name)),
                    layout: texture_bind_group_layout,
                    entries: &[
                        BindGroupEntry {
                            binding: 0,
                            resource: BindingResource::TextureView(&texture_view),
                        },
                        BindGroupEntry {
                            binding: 1,
                            resource: BindingResource::Sampler(&sampler),
                        },
                    ],
                });

                // Set up mip maps
                self.mipmapper.blit_mipmaps(device, queue, &texture);

                // Get the index
                let bind_group_index = self.bind_groups.len();

                self.loaded_textures
                    .insert(texture_file_name.to_owned(), bind_group_index);
                self.bind_groups.push(bind_group);

                Ok(TextureHandle {
                    index: bind_group_index,
                })
            }
        }
    }

    fn get_bind_group(&self, handle: &TextureHandle) -> Option<&BindGroup> {
        self.bind_groups.get(handle.index)
    }
}
