//= IMPORTS ==================================================================

use crate::texture::{TextureHandler, RESULT_TEX_FORMAT};
use crate::Buffers;

use glam::U16Vec2;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, BufferBindingType, ColorTargetState,
    ColorWrites, CommandEncoder, ComputePassDescriptor, ComputePipeline, ComputePipelineDescriptor,
    Device, FragmentState, LoadOp, MultisampleState, Operations, PipelineCompilationOptions,
    PipelineLayoutDescriptor, PrimitiveState, RenderPassColorAttachment, RenderPassDescriptor,
    RenderPipeline, RenderPipelineDescriptor, SamplerBindingType, ShaderModuleDescriptor,
    ShaderSource, ShaderStages, StorageTextureAccess, StoreOp, TextureFormat, TextureSampleType,
    TextureView, TextureViewDimension, VertexState,
};

//= CONSTANTS ================================================================

static RAY_TRACER_SRC: &str = include_str!("shaders/ray_tracer.wgsl");
static PATH_TRACER_SRC: &str = include_str!("shaders/path_tracer.wgsl");
static DENOISER_SRC: &str = include_str!("shaders/denoiser.wgsl");
static SCREEN_SHADER_SRC: &str = include_str!("shaders/screen.wgsl");

//= BINDING TYPES ============================================================

fn uniform_binding_type() -> BindingType {
    BindingType::Buffer {
        ty: BufferBindingType::Uniform,
        has_dynamic_offset: false,
        min_binding_size: None,
    }
}

fn storage_binding_type(read_only: bool) -> BindingType {
    BindingType::Buffer {
        ty: BufferBindingType::Storage { read_only },
        has_dynamic_offset: false,
        min_binding_size: None,
    }
}

//= MACROS ===================================================================

macro_rules! bind_group_layout_entries {
    ($($binding:expr=>($vis:ident)$entry:expr),*$(,)?) => {{[
        $(BindGroupLayoutEntry {
            binding: $binding,
            visibility: ShaderStages::$vis,
            ty: $entry,
            count: None,
        }),*
    ]}}
}

macro_rules! bind_group_entries {
    ($($binding:expr=>$entry:expr),*$(,)?) => {{[
        $(BindGroupEntry {
            binding: $binding,
            resource: $entry,
        }),*
    ]}}
}

//= RAY TRACER (COMPUTE) SHADER ==============================================

pub struct RayTracerShader {
    pub pipeline: ComputePipeline,
    pub bind_group_layout: BindGroupLayout,
    pub bind_group: BindGroup,
}

impl RayTracerShader {
    pub(crate) fn new(
        device: &Device,
        norm_tex: &TextureHandler,
        buffers: &Buffers,
    ) -> Self {
        let shader_module = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("#raytracer.shader-module"),
            source: ShaderSource::Wgsl(RAY_TRACER_SRC.into()),
        });
        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("#raytracer.bind-group-layout"),
            entries: &bind_group_layout_entries!(
                0 => (COMPUTE) BindingType::StorageTexture {
                    access: StorageTextureAccess::WriteOnly,
                    format: RESULT_TEX_FORMAT,
                    view_dimension: TextureViewDimension::D2,
                },
                1 => (COMPUTE) uniform_binding_type(),
                2 => (COMPUTE) storage_binding_type(true),
                3 => (COMPUTE) uniform_binding_type(),
            ),
        });
        let bind_group =
            Self::create_bind_group(device, &bind_group_layout, norm_tex, buffers);

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("#raytracer.pipeline-layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });
        let pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("#raytracer.pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader_module,
            entry_point: Some("update"),
            compilation_options: PipelineCompilationOptions::default(),
            cache: None,
        });

        Self {
            pipeline,
            bind_group,
            bind_group_layout,
        }
    }

    pub(crate) fn create_bind_group(
        device: &Device,
        layout: &BindGroupLayout,
        norm_tex: &TextureHandler,
        buffers: &Buffers,
    ) -> BindGroup {
        device.create_bind_group(&BindGroupDescriptor {
            label: Some("#raytracer.bind-group"),
            layout,
            entries: &bind_group_entries!(
                0 => BindingResource::TextureView(&norm_tex.view),
                1 => buffers.camera_buffer.0.as_entire_binding(),
                2 => buffers.nodes.buf.as_entire_binding(),
                3 => buffers.world_data.0.as_entire_binding(),
            ),
        })
    }

    pub(crate) fn recreate_bind_group(
        &mut self,
        device: &Device,
        norm_tex: &TextureHandler,
        buffers: &Buffers,
    ) {
        self.bind_group =
            Self::create_bind_group(device, &self.bind_group_layout, norm_tex, buffers);
    }

    pub fn encode_pass(&self, encoder: &mut CommandEncoder, workgroups: U16Vec2) {
        let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor {
            label: Some("#raytracer-pass"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.dispatch_workgroups(workgroups.x as u32, workgroups.y as u32, 1);
    }
}

//= PATH TRACER (COMPUTE) SHADER =============================================

pub struct PathTracerShader {
    pub pipeline: ComputePipeline,
    pub bind_group_layout: BindGroupLayout,
    pub bind_group: BindGroup,
}

impl PathTracerShader {
    pub(crate) fn new(
        device: &Device,
        res_tex: &TextureHandler,
        prev_tex: &TextureHandler,
        buffers: &Buffers,
    ) -> Self {
        let shader_module = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("#pathtracer.shader-module"),
            source: ShaderSource::Wgsl(PATH_TRACER_SRC.into()),
        });
        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("#pathtracer.bind-group-layout"),
            entries: &bind_group_layout_entries!(
                0 => (COMPUTE) BindingType::StorageTexture {
                    access: StorageTextureAccess::WriteOnly,
                    format: RESULT_TEX_FORMAT,
                    view_dimension: TextureViewDimension::D2,
                },
                1 => (COMPUTE) uniform_binding_type(),
                2 => (COMPUTE) uniform_binding_type(),
                3 => (COMPUTE) storage_binding_type(true),
                4 => (COMPUTE) storage_binding_type(true),
                5 => (COMPUTE) uniform_binding_type(),
                6 => (COMPUTE) uniform_binding_type(),
                7 => (COMPUTE) BindingType::Texture {
                    sample_type: TextureSampleType::default(),
                    view_dimension: TextureViewDimension::D2,
                    multisampled: false,
                }
            ),
        });
        let bind_group =
            Self::create_bind_group(device, &bind_group_layout, res_tex, prev_tex, buffers);

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("#pathtracer.pipeline-layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });
        let pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("#pathtracer.pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader_module,
            entry_point: Some("update"),
            compilation_options: PipelineCompilationOptions::default(),
            cache: None,
        });

        Self {
            pipeline,
            bind_group,
            bind_group_layout,
        }
    }

    pub(crate) fn create_bind_group(
        device: &Device,
        layout: &BindGroupLayout,
        res_tex: &TextureHandler,
        prev_tex: &TextureHandler,
        buffers: &Buffers,
    ) -> BindGroup {
        device.create_bind_group(&BindGroupDescriptor {
            label: Some("#pathtracer.bind-group"),
            layout,
            entries: &bind_group_entries!(
                0 => BindingResource::TextureView(&res_tex.view),
                1 => buffers.camera_buffer.0.as_entire_binding(),
                2 => buffers.settings.0.as_entire_binding(),
                3 => buffers.nodes.buf.as_entire_binding(),
                4 => buffers.voxel_materials.0.as_entire_binding(),
                5 => buffers.frame_count.0.as_entire_binding(),
                6 => buffers.world_data.0.as_entire_binding(),
                7 => BindingResource::TextureView(&prev_tex.view),
            ),
        })
    }

    pub(crate) fn recreate_bind_group(
        &mut self,
        device: &Device,
        res_tex: &TextureHandler,
        prev_tex: &TextureHandler,
        buffers: &Buffers,
    ) {
        self.bind_group =
            Self::create_bind_group(device, &self.bind_group_layout, res_tex, prev_tex, buffers);
    }

    pub fn encode_pass(&self, encoder: &mut CommandEncoder, workgroups: U16Vec2) {
        let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor {
            label: Some("#pathtracer-pass"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.dispatch_workgroups(workgroups.x as u32, workgroups.y as u32, 1);
    }
}

//= DENOISER (COMPUTE) SHADER ================================================

pub struct DenoiserShader {
    pub pipeline: ComputePipeline,
    pub bind_group_layout: BindGroupLayout,
    pub bind_group: BindGroup,
}

impl DenoiserShader {
    pub(crate) fn new(device: &Device, res_tex: &TextureHandler, den_tex: &TextureHandler) -> Self {
        let shader_module = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("#denoiser.shader-module"),
            source: ShaderSource::Wgsl(DENOISER_SRC.into()),
        });
        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("#denoiser.bind-group-layout"),
            entries: &bind_group_layout_entries!(
                0 => (COMPUTE) BindingType::Texture {
                    sample_type: TextureSampleType::default(),
                    view_dimension: TextureViewDimension::D2,
                    multisampled: false,
                },
                1 => (COMPUTE) BindingType::StorageTexture {
                    access: StorageTextureAccess::WriteOnly,
                    format: RESULT_TEX_FORMAT,
                    view_dimension: TextureViewDimension::D2,
                }
            ),
        });
        let bind_group = Self::create_bind_group(device, &bind_group_layout, res_tex, den_tex);

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("#denoiser.pipeline-layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });
        let pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("#denoiser.pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader_module,
            entry_point: Some("update"),
            compilation_options: PipelineCompilationOptions::default(),
            cache: None,
        });

        Self {
            pipeline,
            bind_group,
            bind_group_layout,
        }
    }

    pub(crate) fn create_bind_group(
        device: &Device,
        layout: &BindGroupLayout,
        res_tex: &TextureHandler,
        den_tex: &TextureHandler,
    ) -> BindGroup {
        device.create_bind_group(&BindGroupDescriptor {
            label: Some("#denoiser.bind-group"),
            layout,
            entries: &bind_group_entries!(
                0 => BindingResource::TextureView(&res_tex.view),
                1 => BindingResource::TextureView(&den_tex.view),
            ),
        })
    }

    pub(crate) fn recreate_bind_group(
        &mut self,
        device: &Device,
        res_tex: &TextureHandler,
        den_tex: &TextureHandler,
    ) {
        self.bind_group =
            Self::create_bind_group(device, &self.bind_group_layout, res_tex, den_tex);
    }

    pub fn encode_pass(&self, encoder: &mut CommandEncoder, workgroups: U16Vec2) {
        let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor {
            label: Some("#denoiser-pass"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.dispatch_workgroups(workgroups.x as u32, workgroups.y as u32, 1);
    }
}

//= SCREEN (FRAGMENT) SHADER =================================================

pub(crate) struct ScreenShader {
    pub(crate) pipeline: RenderPipeline,
    pub(crate) bind_group_layout: BindGroupLayout,
    pub(crate) bind_group: BindGroup,
}

impl ScreenShader {
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
            entries: &bind_group_layout_entries!(
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
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
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
            multiview: None,
            cache: None,
        });

        Self {
            pipeline,
            bind_group,
            bind_group_layout,
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
            entries: &bind_group_entries!(
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
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Load,
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.draw(0..6, 0..1);
    }
}
