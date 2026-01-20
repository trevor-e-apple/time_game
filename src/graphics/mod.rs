use std::{env, fs::File, io::Read, mem, path::Path, sync::Arc};

use crate::{camera::Camera, texture::Texture};

use anyhow::Context;
use cgmath::{Matrix3, Matrix4, Point3, Quaternion, SquareMatrix, Vector2, Vector3};
use image::GenericImageView;
use wgpu::{
    AddressMode, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, BlendState, BufferBindingType,
    BufferDescriptor, BufferUsages, ColorTargetState, ColorWrites, CommandEncoderDescriptor,
    CompareFunction, DepthBiasState, DepthStencilState, Extent3d, Face, FilterMode, FragmentState,
    FrontFace, IndexFormat, LoadOp, MultisampleState, Origin3d, PipelineCompilationOptions,
    PipelineLayoutDescriptor, PolygonMode, PowerPreference, PrimitiveState, PrimitiveTopology,
    RenderPassColorAttachment, RenderPassDepthStencilAttachment, RenderPassDescriptor,
    RenderPipeline, RenderPipelineDescriptor, RequestAdapterOptions, SamplerBindingType,
    ShaderModule, ShaderModuleDescriptor, ShaderSource, ShaderStages, StencilState, StoreOp,
    Surface, SurfaceConfiguration, TexelCopyBufferLayout, TexelCopyTextureInfo, TextureAspect,
    TextureDimension, TextureFormat, TextureSampleType, TextureUsages, TextureView,
    TextureViewDescriptor, TextureViewDimension, VertexAttribute, VertexBufferLayout, VertexFormat,
    VertexState, VertexStepMode,
    util::{BufferInitDescriptor, DeviceExt},
    wgt::{SamplerDescriptor, TextureDescriptor},
};
use winit::window::Window;

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

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct DebugVertex2 {
    pub position: [f32; 2],
    pub color: [f32; 3],
}

impl DebugVertex2 {
    pub fn buffer_layout() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<DebugVertex2>() as wgpu::BufferAddress,
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
                    format: VertexFormat::Float32x3,
                },
            ],
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex3 {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
}

impl Vertex3 {
    pub fn buffer_layout() -> wgpu::VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex3>() as wgpu::BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &[
                VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: VertexFormat::Float32x3,
                },
                VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: VertexFormat::Float32x2,
                },
            ],
        }
    }
}

// TODO: delete triangle vertices? or move to debug-exclusive code?
pub const TRIANGLE_VERTICES: &[Vertex3] = &[
    Vertex3 {
        position: [0.0, 0.5, 0.0],
        tex_coords: [0.0, 0.0], // Debug code, not currently set
    },
    Vertex3 {
        position: [-0.5, -0.5, 0.0],
        tex_coords: [0.0, 0.0],
    },
    Vertex3 {
        position: [0.5, -0.5, 0.0],
        tex_coords: [0.0, 0.0],
    },
];

const DEBUG_TRIANGLE_VERTICES: &[DebugVertex2] = &[
    DebugVertex2 {
        position: [0.0, 0.5],
        color: [0.0, 1.0, 0.0],
    },
    DebugVertex2 {
        position: [-0.5, -0.5],
        color: [0.0, 1.0, 0.0],
    },
    DebugVertex2 {
        position: [0.5, -0.5],
        color: [0.0, 1.0, 0.0],
    },
];

// TODO: delete triangle indeices
pub const TRIANGLE_INDICES: &[u32] = &[0, 1, 2];

pub const SQUARE_VERTICES: &[Vertex3] = &[
    Vertex3 {
        position: [-0.5, 0.5, 0.0],
        tex_coords: [0.0, 0.0],
    },
    Vertex3 {
        position: [0.5, -0.5, 0.0],
        tex_coords: [1.0, 1.0],
    },
    Vertex3 {
        position: [0.5, 0.5, 0.0],
        tex_coords: [1.0, 0.0],
    },
    Vertex3 {
        position: [-0.5, -0.5, 0.0],
        tex_coords: [0.0, 1.0],
    },
];

const DEBUG_SQUARE_VERTICES: &[DebugVertex2] = &[
    DebugVertex2 {
        position: [-0.5, 0.5],
        color: [1.0, 0.0, 0.0],
    },
    DebugVertex2 {
        position: [0.5, -0.5],
        color: [1.0, 0.0, 0.0],
    },
    DebugVertex2 {
        position: [0.5, 0.5],
        color: [1.0, 0.0, 0.0],
    },
    DebugVertex2 {
        position: [-0.5, -0.5],
        color: [1.0, 0.0, 0.0],
    },
];

pub const SQUARE_INDICES: &[u32] = &[0, 1, 2, 0, 3, 1];

const MAX_DEBUG_SQUARES: usize = 1000;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_projection: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view_projection: Matrix4::identity().into(),
        }
    }

    pub fn with_camera(camera: &Camera) -> Self {
        let mut uniform = Self::new();
        uniform.update_view_projection(camera);
        uniform
    }

    // TODO: can we just fold this into with_camera?
    pub fn update_view_projection(&mut self, camera: &Camera) {
        self.view_projection = camera.build_view_projection_matrix().into();
    }
}

pub struct Instance {
    pub position: Vector3<f32>,
    pub scale: Vector3<f32>,
    pub rotation: Quaternion<f32>, // TODO: since we expect this game to be 2D, do we need full Quaternion support?
}

// TODO: does this need to be public?
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct InstanceRaw {
    model: [[f32; 4]; 4],
}

impl InstanceRaw {
    pub fn buffer_layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

impl Instance {
    pub fn to_raw(&self) -> InstanceRaw {
        InstanceRaw {
            model: (Matrix4::from_translation(self.position)
                * Matrix4::from(self.rotation)
                * Matrix4::from_nonuniform_scale(self.scale.x, self.scale.y, self.scale.z))
            .into(),
        }
    }
}

pub struct Instance2D {
    pub position: Vector2<f32>,
    pub scale: Vector2<f32>,
    pub rotation: cgmath::Rad<f32>,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Instance2DRaw {
    model: [[f32; 3]; 3],
}

impl Instance2DRaw {
    pub fn buffer_layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Instance2DRaw>() as wgpu::BufferAddress,
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

impl Instance2D {
    fn to_raw(&self) -> Instance2DRaw {
        Instance2DRaw {
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

struct DebugSquare {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    instance_buffer: wgpu::Buffer,
    num_instances: u32,
}

struct DebugTriangle {
    vertex_buffer: wgpu::Buffer,
    instance_buffer: wgpu::Buffer,
    num_instances: u32,
}

const MAX_DEBUG_TRIANGLES: usize = 1000;

///
fn load_shader(device: &wgpu::Device, shader_file_name: &str, shader_label: &str) -> ShaderModule {
    let shader_source_dir = env::var("SHADER_SOURCE_DIR").unwrap();
    let shader_path = Path::new(&shader_source_dir).join(shader_file_name);
    let mut shader_source_file = File::open(shader_path).unwrap();

    let mut shader_source_string = String::new();
    shader_source_file
        .read_to_string(&mut shader_source_string)
        .unwrap();

    device.create_shader_module(ShaderModuleDescriptor {
        label: Some(shader_label),
        source: ShaderSource::Wgsl(shader_source_string.into()),
    })
}

pub struct GraphicsState {
    surface: Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: SurfaceConfiguration,
    render_pipeline: RenderPipeline,
    debug_pipeline: RenderPipeline,
    debug_triangle: DebugTriangle,
    debug_square: DebugSquare,
    models: Vec<Model>,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: BindGroup,
    pub camera: Camera,
    depth_texture: Texture,
    diffuse_bind_group: BindGroup,
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

        // Need the size for the surface configuration
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

        let camera = Camera {
            eye: Point3::new(0.0, 0.0, 2.0),
            target: Point3::new(0.0, 0.0, 0.0),
            up: Vector3::new(0.0, 1.0, 0.0),
            aspect: config.width as f32 / config.height as f32,
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,
        };

        let camera_uniform = CameraUniform::with_camera(&camera);

        let camera_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("Camera Bind Group Layout"),
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
            label: Some("Camera Bind Group"),
            layout: &camera_bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

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
                    buffers: &[Vertex3::buffer_layout(), InstanceRaw::buffer_layout()],
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
                depth_stencil: Some(DepthStencilState {
                    format: Texture::DEPTH_FORMAT,
                    depth_write_enabled: true,
                    depth_compare: CompareFunction::Less,
                    stencil: StencilState::default(),
                    bias: DepthBiasState::default(),
                }),
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

        let debug_pipeline = {
            let shader = load_shader(&device, "debug_shader.wgsl", "Debug pipeline shader");

            let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
                label: Some("Debug Pipeline Layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });
            let debug_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
                label: Some("Debug Pipeline"),
                layout: Some(&pipeline_layout),
                vertex: VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    compilation_options: PipelineCompilationOptions::default(),
                    buffers: &[
                        DebugVertex2::buffer_layout(),
                        Instance2DRaw::buffer_layout(),
                    ],
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
                depth_stencil: Some(DepthStencilState {
                    format: Texture::DEPTH_FORMAT,
                    depth_write_enabled: true,
                    depth_compare: CompareFunction::Less,
                    stencil: StencilState::default(),
                    bias: DepthBiasState::default(),
                }),
                multisample: MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
                cache: None,
            });

            debug_pipeline
        };

        let debug_square = {
            let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
                label: Some("Square Vertex Buffer"),
                contents: bytemuck::cast_slice(DEBUG_SQUARE_VERTICES),
                usage: BufferUsages::VERTEX,
            });
            let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
                label: Some("Square Index Buffer"),
                contents: bytemuck::cast_slice(SQUARE_INDICES),
                usage: BufferUsages::INDEX,
            });
            let instance_buffer = device.create_buffer(&BufferDescriptor {
                label: Some("Square Instance Buffer"),
                size: (mem::size_of::<Instance2DRaw>() * MAX_DEBUG_SQUARES) as wgpu::BufferAddress,
                usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });

            DebugSquare {
                vertex_buffer,
                index_buffer,
                instance_buffer,
                num_instances: 0,
            }
        };
        let debug_triangle = {
            let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
                label: Some("Triangle Vertex Buffer"),
                contents: bytemuck::cast_slice(DEBUG_TRIANGLE_VERTICES),
                usage: BufferUsages::VERTEX,
            });
            let instance_buffer = device.create_buffer(&BufferDescriptor {
                label: Some("Triangle Instance Buffer"),
                size: (mem::size_of::<Instance2DRaw>() * MAX_DEBUG_TRIANGLES)
                    as wgpu::BufferAddress,
                usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });

            DebugTriangle {
                vertex_buffer,
                instance_buffer,
                num_instances: 0,
            }
        };

        let depth_texture = Texture::create_depth_texture(&device, &config, "Depth Texture");

        let models = vec![];

        Ok(Self {
            surface,
            device,
            queue,
            config,
            render_pipeline,
            debug_pipeline,
            debug_square,
            debug_triangle,
            camera,
            camera_buffer,
            camera_bind_group,
            models,
            depth_texture,
            diffuse_bind_group,
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        // TODO: is it possible to get zero size?
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);

        self.depth_texture =
            Texture::create_depth_texture(&self.device, &self.config, "Depth Texture");
    }

    pub fn update_camera_buffer(&mut self) {
        let camera_uniform = CameraUniform::with_camera(&self.camera);
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[camera_uniform]),
        );
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
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: LoadOp::Clear(1.0),
                        store: StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            // Draw models
            {
                render_pass.set_pipeline(&self.render_pipeline);
                render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
                render_pass.set_bind_group(1, &self.camera_bind_group, &[]);

                for model in &self.models {
                    render_pass.set_vertex_buffer(0, model.vertex_buffer.slice(..));
                    render_pass.set_index_buffer(model.index_buffer.slice(..), IndexFormat::Uint32);
                    render_pass.set_vertex_buffer(1, model.instance_buffer.slice(..));
                    render_pass.draw_indexed(0..model.num_indices, 0, 0..model.num_instances);
                }
            }

            // Begin debug rendering
            {
                render_pass.set_pipeline(&self.debug_pipeline);

                // Draw debug squares
                {
                    render_pass.set_vertex_buffer(0, self.debug_square.vertex_buffer.slice(..));
                    render_pass.set_index_buffer(
                        self.debug_square.index_buffer.slice(..),
                        IndexFormat::Uint32,
                    );
                    render_pass.set_vertex_buffer(1, self.debug_square.instance_buffer.slice(..));
                    render_pass.draw_indexed(0..6, 0, 0..self.debug_square.num_instances);
                }

                // Draw debug triangle
                {
                    render_pass.set_vertex_buffer(0, self.debug_triangle.vertex_buffer.slice(..));
                    render_pass.set_vertex_buffer(1, self.debug_triangle.instance_buffer.slice(..));
                    render_pass.draw(0..3, 0..self.debug_triangle.num_instances);
                }
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        Ok(())
    }

    pub fn add_model(
        &mut self,
        vertices: &[Vertex3],
        indices: &[u32],
        max_instances: usize,
    ) -> usize {
        // TODO: Have a way to provide labels
        let vertex_buffer = self.device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(vertices),
            usage: BufferUsages::VERTEX,
        });
        let index_buffer = self.device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(indices),
            usage: BufferUsages::INDEX,
        });
        let instance_buffer = self.device.create_buffer(&BufferDescriptor {
            label: Some("Instance Buffer"),
            size: (mem::size_of::<InstanceRaw>() * max_instances) as wgpu::BufferAddress,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let model_index = self.models.len();
        self.models.push(Model {
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
    pub fn add_instance(&mut self, model_index: usize, instance: Instance) {
        if let Some(model) = self.models.get_mut(model_index) {
            assert!(
                (model.num_instances as usize) < model.max_instances,
                "Exceeded maximum number of instances for model"
            );

            self.queue.write_buffer(
                &model.instance_buffer,
                (model.num_instances as usize * mem::size_of::<InstanceRaw>())
                    as wgpu::BufferAddress,
                bytemuck::cast_slice(&[instance.to_raw()]),
            );
            model.num_instances += 1;
        }
    }

    pub fn add_debug_square(&mut self, instance: Instance2D) {
        self.queue.write_buffer(
            &self.debug_square.instance_buffer,
            (self.debug_square.num_instances as usize * mem::size_of::<Instance2DRaw>())
                as wgpu::BufferAddress,
            bytemuck::cast_slice(&[instance.to_raw()]),
        );
        self.debug_square.num_instances += 1;
    }

    pub fn add_debug_triangle(&mut self, instance: Instance2D) {
        self.queue.write_buffer(
            &self.debug_triangle.instance_buffer,
            (self.debug_triangle.num_instances as usize * mem::size_of::<Instance2DRaw>())
                as wgpu::BufferAddress,
            bytemuck::cast_slice(&[instance.to_raw()]),
        );
        self.debug_triangle.num_instances += 1;
    }
}
