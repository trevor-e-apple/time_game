use std::mem;

use cgmath::{Matrix3, Vector2};
use wgpu::{VertexAttribute, VertexBufferLayout, VertexFormat, VertexStepMode};

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

pub const DEBUG_TRIANGLE_VERTICES: &[DebugVertex2] = &[
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

pub const DEBUG_SQUARE_VERTICES: &[DebugVertex2] = &[
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

pub struct Instance2D {
    pub position: Vector2<f32>,
    pub scale: Vector2<f32>,
    pub rotation: cgmath::Rad<f32>,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Instance2DRaw {
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
    pub fn to_raw(&self) -> Instance2DRaw {
        Instance2DRaw {
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
