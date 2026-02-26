//= IMPORTS ========================================================================================

use glam::U16Vec2;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, CommandEncoder, ComputePassDescriptor,
    ComputePipeline, ComputePipelineDescriptor, Device, PipelineCompilationOptions,
    PipelineLayoutDescriptor, ShaderModuleDescriptor, ShaderSource, ShaderStages,
    StorageTextureAccess, TextureSampleType, TextureViewDimension,
};

use crate::buffers::Buffers;
use crate::passes::{PATH_TRACER_SRC, storage_binding_type, uniform_binding_type};
use crate::texture::{RESULT_TEX_FORMAT, TextureHandler};

//= PATH TRACER (COMPUTE) SHADER =============================================

pub(crate) struct PathTracerPass {
    pub pipeline: ComputePipeline,
    pub bind_group_layout: BindGroupLayout,
    pub bind_group: BindGroup,
}

impl PathTracerPass {
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
            entries: &crate::bind_group_layout_entries!(
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
            immediate_size: 0,
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
            bind_group_layout,
            bind_group,
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
            entries: &crate::bind_group_entries!(
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
        pass.dispatch_workgroups(u32::from(workgroups.x), u32::from(workgroups.y), 1);
    }
}
