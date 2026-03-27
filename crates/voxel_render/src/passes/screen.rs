//= IMPORTS ========================================================================================

use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, ColorTargetState, ColorWrites,
    CommandEncoder, Device, FragmentState, LoadOp, MultisampleState, Operations,
    PipelineCompilationOptions, PipelineLayoutDescriptor, PrimitiveState,
    RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor,
    SamplerBindingType, ShaderModuleDescriptor, ShaderSource, ShaderStages, StoreOp, TextureFormat,
    TextureSampleType, TextureView, TextureViewDimension, VertexState,
};

use crate::passes::SCREEN_SHADER_SRC;
use crate::texture::TextureHandler;

//= SCREEN (FRAGMENT) SHADER =================================================

pub(crate) struct ScreenPass {
    pub(crate) pipeline: RenderPipeline,
    pub(crate) bind_group_layout: BindGroupLayout,
    pub(crate) bind_group: BindGroup,
}

impl ScreenPass {
    pub(crate) fn new(
        device: &Device,
        tex: &TextureHandler,
        surface_format: TextureFormat,
    ) -> Self {
        let shader_module = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("output-tex-shader.shader-module"),
            source: ShaderSource::Wgsl(SCREEN_SHADER_SRC.into()),
        });

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("output-tex-shader.bind-group-layout"),
            entries: &crate::bind_group_layout_entries!(
                0 => (FRAGMENT) BindingType::Texture {
                    sample_type: TextureSampleType::default(),
                    view_dimension: TextureViewDimension::D2,
                    multisampled: false,
                },
                1 => (FRAGMENT) BindingType::Sampler(SamplerBindingType::Filtering),
            ),
        });
        let bind_group = Self::create_bind_group(device, &bind_group_layout, tex);

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("output-tex-shader.pipeline-layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("output-tex-shader.pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader_module,
                entry_point: Some("vs_main"),
                compilation_options: PipelineCompilationOptions::default(),
                buffers: &[],
            },
            fragment: Some(FragmentState {
                module: &shader_module,
                entry_point: Some("fs_main"),
                compilation_options: PipelineCompilationOptions::default(),
                targets: &[Some(ColorTargetState {
                    format: surface_format,
                    blend: None,
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState::default(),
            depth_stencil: None,
            multisample: MultisampleState::default(),
            cache: None,
            multiview_mask: None,
        });

        Self {
            pipeline,
            bind_group_layout,
            bind_group,
        }
    }

    pub(crate) fn create_bind_group(
        device: &Device,
        layout: &BindGroupLayout,
        tex: &TextureHandler,
    ) -> BindGroup {
        device.create_bind_group(&BindGroupDescriptor {
            label: Some("output-tex-shader.bind_group"),
            layout,
            entries: &crate::bind_group_entries!(
                0 => BindingResource::TextureView(&tex.view),
                1 => BindingResource::Sampler(&tex.sampler),
            ),
        })
    }

    pub(crate) fn recreate_bind_group(&mut self, device: &Device, tex: &TextureHandler) {
        self.bind_group = Self::create_bind_group(device, &self.bind_group_layout, tex);
    }

    pub(crate) fn encode_pass(&self, encoder: &mut CommandEncoder, view: &TextureView) {
        let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("#output-tex-shader-pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view,
                depth_slice: None,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Load,
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.draw(0..6, 0..1);
    }
}
