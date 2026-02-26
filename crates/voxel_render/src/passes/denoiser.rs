//= IMPORTS ========================================================================================

use glam::U16Vec2;
use wgpu::{BindGroupLayoutEntry, BindGroup, BindGroupEntry, ShaderStages, BindGroupDescriptor, BindGroupLayout, BindGroupLayoutDescriptor, BindingType, CommandEncoder, ComputePassDescriptor, ComputePipeline, ComputePipelineDescriptor, Device, PipelineCompilationOptions, PipelineLayoutDescriptor, ShaderModuleDescriptor, ShaderSource, StorageTextureAccess, TextureSampleType, TextureViewDimension, BindingResource};

use crate::bind_group_layout_entries;
use crate::passes::DENOISER_SRC;
use crate::texture::{TextureHandler, RESULT_TEX_FORMAT};

//= DENOISER (COMPUTE) SHADER ================================================

pub(crate) struct DenoiserPass {
	pub pipeline: ComputePipeline,
	pub bind_group_layout: BindGroupLayout,
	pub bind_group: BindGroup,
}

impl DenoiserPass {
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
			immediate_size: 0,
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
			bind_group_layout,
			bind_group,
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
			entries: &crate::bind_group_entries!(
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
		pass.dispatch_workgroups(u32::from(workgroups.x), u32::from(workgroups.y), 1);
	}
}
