use anyhow::bail;
use wgpu::{
    BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, ColorTargetState, ColorWrites, Face,
    FilterMode, FragmentState, FrontFace, MultisampleState, PipelineCompilationOptions,
    PipelineLayoutDescriptor, PolygonMode, PrimitiveState, PrimitiveTopology,
    RenderPassColorAttachment, RenderPipeline, RenderPipelineDescriptor, SamplerBindingType,
    SamplerDescriptor, ShaderStages, TexelCopyTextureInfo, TextureDescriptor, TextureFormat,
    TextureSampleType, TextureUsages, TextureViewDescriptor, TextureViewDimension, VertexState,
};

use crate::graphics::shader::load_shader;

pub struct Mipmapper {
    render_pipeline: RenderPipeline,
    sampler: wgpu::Sampler,
    storage_texture_layout: BindGroupLayout,
}

impl Mipmapper {
    pub fn new(device: &wgpu::Device) -> Self {
        let shader = load_shader(device, "blit.wgsl", "blit shader");
        let blit_format = TextureFormat::Rgba8Unorm;

        let texture_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("Mipmapper Texture Bind Group Layout"),
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

        let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Mipmapper Pipeline Layout"),
            bind_group_layouts: &[&texture_bind_group_layout],
            immediate_size: 0,
        });

        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Mipmapper Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: PipelineCompilationOptions::default(),
                buffers: &[],
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: Some("fs_color"),
                compilation_options: PipelineCompilationOptions::default(),
                targets: &[Some(ColorTargetState {
                    format: blit_format,
                    blend: None,
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
        let sampler = device.create_sampler(&SamplerDescriptor {
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            ..Default::default()
        });

        Self {
            render_pipeline,
            sampler,
            storage_texture_layout: texture_bind_group_layout,
        }
    }

    pub fn blit_mipmaps(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture: &wgpu::Texture,
    ) -> anyhow::Result<()> {
        if texture.format() != TextureFormat::Rgba8Unorm
            && texture.format() != TextureFormat::Rgba8UnormSrgb
        {
            bail!("Unsupported texture format");
        }

        if texture.mip_level_count() == 1 {
            return Ok(());
        }

        let mut encoder = device.create_command_encoder(&Default::default());

        let (mut src_view, maybe_temp) =
            if texture.usage().contains(TextureUsages::RENDER_ATTACHMENT) {
                (
                    texture.create_view(&TextureViewDescriptor {
                        format: Some(texture.format().remove_srgb_suffix()),
                        base_mip_level: 0,
                        mip_level_count: Some(1),
                        ..Default::default()
                    }),
                    None,
                )
            } else {
                let temp = device.create_texture(&TextureDescriptor {
                    label: Some("Mipmapper temp texture"),
                    size: texture.size(),
                    mip_level_count: texture.mip_level_count(),
                    sample_count: texture.sample_count(),
                    dimension: texture.dimension(),
                    format: texture.format().remove_srgb_suffix(),
                    usage: TextureUsages::RENDER_ATTACHMENT
                        | TextureUsages::TEXTURE_BINDING
                        | TextureUsages::COPY_DST
                        | TextureUsages::COPY_SRC,
                    view_formats: &[],
                });
                encoder.copy_texture_to_texture(
                    texture.as_image_copy(),
                    temp.as_image_copy(),
                    temp.size(),
                );
                (
                    temp.create_view(&TextureViewDescriptor {
                        mip_level_count: Some(1),
                        ..Default::default()
                    }),
                    Some(temp),
                )
            };

        for mip in 1..texture.mip_level_count() {
            let dst_view = src_view.texture().create_view(&TextureViewDescriptor {
                format: Some(texture.format().remove_srgb_suffix()),
                base_mip_level: mip,
                mip_level_count: Some(1),
                ..Default::default()
            });

            let texture_bind_group = device.create_bind_group(&BindGroupDescriptor {
                label: None,
                layout: &self.render_pipeline.get_bind_group_layout(0),
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: BindingResource::TextureView(&src_view),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: BindingResource::Sampler(&self.sampler),
                    },
                ],
            });

            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &dst_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&self.render_pipeline);
            pass.set_bind_group(0, &texture_bind_group, &[]);
            pass.draw(0..3, 0..1);

            src_view = dst_view;
        }

        // If we created a temporary texture, we now need to copy it back to the original
        if let Some(temp) = maybe_temp {
            let mut size = temp.size();
            for mip_level in 0..temp.mip_level_count() {
                encoder.copy_texture_to_texture(
                    TexelCopyTextureInfo {
                        mip_level,
                        ..temp.as_image_copy()
                    },
                    TexelCopyTextureInfo {
                        mip_level,
                        ..texture.as_image_copy()
                    },
                    size,
                );

                size.width /= 2;
                size.height /= 2;
            }
        }

        queue.submit([encoder.finish()]);

        Ok(())
    }
}
