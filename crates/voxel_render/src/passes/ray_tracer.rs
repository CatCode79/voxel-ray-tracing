//= IMPORTS ========================================================================================

use glam::U16Vec2;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, CommandEncoder, ComputePassDescriptor,
    ComputePipeline, ComputePipelineDescriptor, Device, PipelineCompilationOptions,
    PipelineLayoutDescriptor, ShaderModuleDescriptor, ShaderSource, ShaderStages,
    StorageTextureAccess, TextureViewDimension,
};

use crate::buffers::Buffers;
use crate::passes::{RAY_TRACER_SRC, storage_binding_type, uniform_binding_type};
use crate::texture::{RESULT_TEX_FORMAT, TextureHandler};

//= RAY TRACER (COMPUTE) SHADER ==============================================

pub(crate) struct RayTracerPass {
    pub pipeline: ComputePipeline,
    pub bind_group_layout: BindGroupLayout,
    pub bind_group: BindGroup,
}

impl RayTracerPass {
    pub(crate) fn new(device: &Device, norm_tex: &TextureHandler, buffers: &Buffers) -> Self {
        let shader_module = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("#raytracer.shader-module"),
            source: ShaderSource::Wgsl(RAY_TRACER_SRC.into()),
        });
        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("#raytracer.bind-group-layout"),
            entries: &crate::bind_group_layout_entries!(
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
        let bind_group = Self::create_bind_group(device, &bind_group_layout, norm_tex, buffers);

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("#raytracer.pipeline-layout"),
            bind_group_layouts: &[&bind_group_layout],
            immediate_size: 0,
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
            bind_group_layout,
            bind_group,
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
            entries: &crate::bind_group_entries!(
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
        pass.dispatch_workgroups(u32::from(workgroups.x), u32::from(workgroups.y), 1);
    }
}
