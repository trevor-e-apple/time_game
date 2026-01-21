use std::mem;

use cgmath::{Matrix3, Vector2};
use wgpu::{
    BlendState, BufferDescriptor, BufferUsages, ColorTargetState, ColorWrites, CompareFunction,
    DepthBiasState, DepthStencilState, Device, Face, FragmentState, FrontFace, IndexFormat,
    MultisampleState, PipelineCompilationOptions, PipelineLayoutDescriptor, PolygonMode,
    PrimitiveState, PrimitiveTopology, RenderPass, RenderPipeline, RenderPipelineDescriptor,
    StencilState, SurfaceConfiguration, VertexAttribute, VertexBufferLayout, VertexFormat,
    VertexState, VertexStepMode,
    util::{BufferInitDescriptor, DeviceExt},
};

use crate::graphics::{common_models::SQUARE_INDICES, shader::load_shader, texture};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex2 {
    position: [f32; 2],
    color: [f32; 3],
}

impl Vertex2 {
    fn buffer_layout() -> VertexBufferLayout<'static> {
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
                    format: VertexFormat::Float32x3,
                },
            ],
        }
    }
}

const TRIANGLE_VERTICES: &[Vertex2] = &[
    Vertex2 {
        position: [0.0, 0.5],
        color: [0.0, 1.0, 0.0],
    },
    Vertex2 {
        position: [-0.5, -0.5],
        color: [0.0, 1.0, 0.0],
    },
    Vertex2 {
        position: [0.5, -0.5],
        color: [0.0, 1.0, 0.0],
    },
];

const SQUARE_VERTICES: &[Vertex2] = &[
    Vertex2 {
        position: [-0.5, 0.5],
        color: [1.0, 0.0, 0.0],
    },
    Vertex2 {
        position: [0.5, -0.5],
        color: [1.0, 0.0, 0.0],
    },
    Vertex2 {
        position: [0.5, 0.5],
        color: [1.0, 0.0, 0.0],
    },
    Vertex2 {
        position: [-0.5, -0.5],
        color: [1.0, 0.0, 0.0],
    },
];

pub struct Instance {
    pub position: Vector2<f32>,
    pub scale: Vector2<f32>,
    pub rotation: cgmath::Rad<f32>,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceRaw {
    model: [[f32; 3]; 3],
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

impl Instance {
    pub fn to_raw(&self) -> InstanceRaw {
        InstanceRaw {
            model: (Matrix3::from_translation(self.position)
                * Matrix3::from_angle_z(self.rotation)
                * Matrix3::from_nonuniform_scale(self.scale.x, self.scale.y))
            .into(),
        }
    }
}

pub struct DebugSquare {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub instance_buffer: wgpu::Buffer,
    pub num_instances: u32,
}

pub struct DebugTriangle {
    pub vertex_buffer: wgpu::Buffer,
    pub instance_buffer: wgpu::Buffer,
    pub num_instances: u32,
}

pub struct DebugState {
    pipeline: RenderPipeline,
    debug_triangle: DebugTriangle,
    debug_square: DebugSquare,
}

impl DebugState {
    const MAX_DEBUG_SQUARES: usize = 1000;
    const MAX_DEBUG_TRIANGLES: usize = 1000;

    pub fn new(device: &Device, config: &SurfaceConfiguration) -> Self {
        let pipeline = {
            let shader = load_shader(device, "debug_shader.wgsl", "Debug pipeline shader");

            let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
                label: Some("Debug Pipeline Layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });
            let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
                label: Some("Debug Pipeline"),
                layout: Some(&pipeline_layout),
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
                depth_stencil: Some(DepthStencilState {
                    format: texture::Texture::DEPTH_FORMAT,
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

            pipeline
        };

        let debug_square = {
            let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
                label: Some("Square Vertex Buffer"),
                contents: bytemuck::cast_slice(SQUARE_VERTICES),
                usage: BufferUsages::VERTEX,
            });
            let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
                label: Some("Square Index Buffer"),
                contents: bytemuck::cast_slice(SQUARE_INDICES),
                usage: BufferUsages::INDEX,
            });
            let instance_buffer = device.create_buffer(&BufferDescriptor {
                label: Some("Square Instance Buffer"),
                size: (mem::size_of::<InstanceRaw>() * Self::MAX_DEBUG_SQUARES)
                    as wgpu::BufferAddress,
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
                contents: bytemuck::cast_slice(TRIANGLE_VERTICES),
                usage: BufferUsages::VERTEX,
            });
            let instance_buffer = device.create_buffer(&BufferDescriptor {
                label: Some("Triangle Instance Buffer"),
                size: (mem::size_of::<InstanceRaw>() * Self::MAX_DEBUG_TRIANGLES)
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

        Self {
            pipeline,
            debug_triangle,
            debug_square,
        }
    }

    pub fn render(&self, render_pass: &mut RenderPass<'_>) {
        render_pass.set_pipeline(&self.pipeline);

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

    pub fn add_square(
        &mut self,
        queue: &wgpu::Queue,
        position: Vector2<f32>,
        scale: Vector2<f32>,
        rotation: f32,
    ) {
        let instance = Instance {
            position,
            scale,
            rotation: cgmath::Rad(rotation),
        };
        queue.write_buffer(
            &self.debug_square.instance_buffer,
            (self.debug_square.num_instances as usize * mem::size_of::<InstanceRaw>())
                as wgpu::BufferAddress,
            bytemuck::cast_slice(&[instance.to_raw()]),
        );
        self.debug_square.num_instances += 1;
    }

    pub fn add_triangle(
        &mut self,
        queue: &wgpu::Queue,
        position: Vector2<f32>,
        scale: Vector2<f32>,
        rotation: f32,
    ) {
        let instance = Instance {
            position,
            scale,
            rotation: cgmath::Rad(rotation),
        };
        queue.write_buffer(
            &self.debug_triangle.instance_buffer,
            (self.debug_triangle.num_instances as usize * mem::size_of::<InstanceRaw>())
                as wgpu::BufferAddress,
            bytemuck::cast_slice(&[instance.to_raw()]),
        );
        self.debug_triangle.num_instances += 1;
    }
}
