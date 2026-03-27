//= IMPORTS ==================================================================

use crate::gpu::{
    create_instance, create_surface, create_surface_config, request_adapter, request_device,
};
use crate::passes::{DenoiserPass, PathTracerPass, RayTracerPass, ScreenPass};
use crate::texture::{RESULT_TEX_FORMAT, RESULT_TEX_USAGES, TextureHandler};
use crate::{Buffers, Camera, FrameData, Material, Node, Settings, WorldData};

use glam::U16Vec2;
use raw_window_handle as rwh;
use wgpu::{CommandBuffer, CommandEncoder, CommandEncoderDescriptor, CurrentSurfaceTexture, Device, Limits, Queue, Surface, SurfaceConfiguration, SurfaceTexture, TextureView, TextureViewDescriptor};

use core::num::NonZeroU16;
use std::iter;

//= RENDERER =================================================================

pub struct Renderer {
    surface: Surface<'static>,
    surface_config: SurfaceConfiguration,
    device: Device,
    queue: Queue,

    normal_texture: TextureHandler,
    result_texture: TextureHandler,
    prev_result_texture: TextureHandler,
    denoised_texture: TextureHandler,
    buffers: Buffers,
    frame_data: FrameData,

    ray_tracer_pass: RayTracerPass,
    path_tracer_pass: PathTracerPass,
    denoiser_pass: DenoiserPass,
    screen_pass: ScreenPass,
}

impl Renderer {
    pub fn new(
        raw_display_handle: Result<rwh::RawDisplayHandle, rwh::HandleError>,
        raw_window_handle: Result<rwh::RawWindowHandle, rwh::HandleError>,
        surface_width: u16,
        surface_height: u16,
        max_nodes: u64,
        _sample_count: u8,
    ) -> Result<Self, String> {
        let instance = create_instance();

        let surface = create_surface(raw_display_handle, raw_window_handle, &instance)?;

        let adapter = request_adapter(&instance, &surface)?;

        let surface_config =
            create_surface_config(&surface, &adapter, surface_width, surface_height)?;

        let (device, queue) = request_device(&adapter, Self::max_buffer_sizes())?;

        surface.configure(&device, &surface_config);

        let buffers = Buffers::new(&device, max_nodes);

        let normal_texture = TextureHandler::new(
            &device,
            U16Vec2::new(surface_width, surface_height),
            RESULT_TEX_FORMAT,
            RESULT_TEX_USAGES,
        );

        let result_texture = TextureHandler::new(
            &device,
            U16Vec2::new(surface_width, surface_height),
            RESULT_TEX_FORMAT,
            RESULT_TEX_USAGES,
        );

        let prev_result_texture = TextureHandler::new(
            &device,
            U16Vec2::new(surface_width, surface_height),
            RESULT_TEX_FORMAT,
            RESULT_TEX_USAGES,
        );

        let denoised_texture = TextureHandler::new(
            &device,
            U16Vec2::new(surface_width, surface_height),
            RESULT_TEX_FORMAT,
            RESULT_TEX_USAGES,
        );

        let ray_tracer = RayTracerPass::new(&device, &normal_texture, &buffers);
        let path_tracer =
            PathTracerPass::new(&device, &result_texture, &prev_result_texture, &buffers);
        let denoiser_shader = DenoiserPass::new(&device, &result_texture, &denoised_texture);
        let screen_shader = ScreenPass::new(&device, &denoised_texture, surface_config.format);

        Ok(Self {
            surface,
            surface_config,
            device,
            queue,

            normal_texture,
            result_texture,
            prev_result_texture,
            denoised_texture,
            buffers,
            frame_data: FrameData::default(),

            ray_tracer_pass: ray_tracer,
            path_tracer_pass: path_tracer,
            screen_pass: screen_shader,
            denoiser_pass: denoiser_shader,
        })
    }

    #[must_use]
    pub fn max_buffer_sizes() -> u64 {
        Limits::default().max_storage_buffer_binding_size
    }

    //= ENCODER AND SUBMIT PASS ==============================================

    pub fn create_command_encoder(&self) -> CommandEncoder {
        self.device
            .create_command_encoder(&CommandEncoderDescriptor::default())
    }

    pub fn submit_once(&mut self, command_buffer: CommandBuffer) {
        self.queue.submit(iter::once(command_buffer));
    }

    //= BUFFERS ==============================================================

    pub fn write_camera(&self, camera: &Camera) {
        self.buffers.camera_buffer.write(&self.queue, camera);
    }

    pub fn write_settings(&self, settings: &Settings) {
        self.buffers.settings.write(&self.queue, settings);
    }

    pub fn write_world_data(&self, world_data: &WorldData) {
        self.buffers.world_data.write(&self.queue, world_data);
    }

    pub fn write_nodes(&self, offset: u64, nodes: &[Node]) {
        self.buffers.nodes.write(&self.queue, offset, nodes);
    }

    pub fn write_voxel_materials(&self, offset: u64, voxel_materials: &[Material]) {
        self.buffers
            .voxel_materials
            .write_slice(&self.queue, offset, voxel_materials);
    }

    pub(crate) fn write_frame_data(&self, frame_data: &FrameData) {
        self.buffers.frame_count.write(&self.queue, frame_data);
    }

    //= RESIZE AND SCALE =====================================================

    pub fn resize(&mut self, width: NonZeroU16, height: NonZeroU16) {
        let width = width.get();
        let height = height.get();
        if self.surface_config.width as u16 != width || self.surface_config.height as u16 != height
        {
            self.surface_config.width = u32::from(width);
            self.surface_config.height = u32::from(height);
            self.surface.configure(&self.device, &self.surface_config);

            let new_size = U16Vec2::new(width, height);
            self.normal_texture =
                TextureHandler::new(&self.device, new_size, RESULT_TEX_FORMAT, RESULT_TEX_USAGES);
            self.result_texture =
                TextureHandler::new(&self.device, new_size, RESULT_TEX_FORMAT, RESULT_TEX_USAGES);
            self.prev_result_texture =
                TextureHandler::new(&self.device, new_size, RESULT_TEX_FORMAT, RESULT_TEX_USAGES);
            self.denoised_texture =
                TextureHandler::new(&self.device, new_size, RESULT_TEX_FORMAT, RESULT_TEX_USAGES);

            self.ray_tracer_pass.recreate_bind_group(
                &self.device,
                &self.normal_texture,
                &self.buffers,
            );
            self.path_tracer_pass.recreate_bind_group(
                &self.device,
                &self.result_texture,
                &self.prev_result_texture,
                &self.buffers,
            );
            self.denoiser_pass.recreate_bind_group(
                &self.device,
                &self.result_texture,
                &self.denoised_texture,
            );
            self.screen_pass
                .recreate_bind_group(&self.device, &self.denoised_texture);
        }

        self.reset_frame_counter();
    }

    /*
            pub fn scale(&mut self, factor: f64) {
            let width = match NonZeroU32::new((self.surface_width as f64 / factor) as u32) {
                None => {
                    log::warn!("[scale] surface_size.width is zero");
                    NonZeroU32::new(1).unwrap()
                }
                Some(w) => w,
            };
            let height = match NonZeroU32::new((self.surface_height as f64 / factor) as u32) {
                None => {
                    log::warn!("[scale] surface_size.height is zero");
                    NonZeroU32::new(1).unwrap()
                }
                Some(h) => h,
            };
            self.resize(width, height)
        }
    */

    //= UPDATE ===============================================================

    pub const fn reset_frame_counter(&mut self) {
        self.frame_data.reset();
    }

    pub fn update(&mut self, camera: Camera) -> Result<(), String> {
        profiling::scope!("Renderer.update()");
        let (output_texture, output_view) = self.get_output();
        let surface_size = self.surface_size();
        let mut encoder = self.create_command_encoder();

        {
            self.write_frame_data(&self.frame_data);
            self.frame_data.increment();
            self.write_camera(&camera);
        }

        let workgroups = surface_size / 8;
        self.ray_tracer_pass.encode_pass(&mut encoder, workgroups);
        self.path_tracer_pass.encode_pass(&mut encoder, workgroups);
        self.denoiser_pass.encode_pass(&mut encoder, workgroups);
        self.screen_pass.encode_pass(&mut encoder, &output_view);

        encoder.copy_texture_to_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.result_texture.handle,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyTextureInfo {
                texture: &self.prev_result_texture.handle,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::Extent3d {
                width: u32::from(self.result_texture.size().x),
                height: u32::from(self.result_texture.size().y),
                depth_or_array_layers: 1,
            },
        );

        self.submit_once(encoder.finish());

        profiling::scope!("output_texture.present()");
        {
            output_texture.present();
        }

        Ok(())
    }

    //= SURFACE TEXTURE ======================================================

    pub const fn surface_size(&self) -> U16Vec2 {
        U16Vec2::new(
            self.surface_config.width as u16,
            self.surface_config.height as u16,
        )
    }

    pub fn get_output(&self) -> (SurfaceTexture, TextureView) {
        fn panic_error(current: CurrentSurfaceTexture) -> ! {
            panic!("Failed to get the current texture: {current:?}");
        }

        let output = match self.surface.get_current_texture() {
            CurrentSurfaceTexture::Success(o) => o,
            CurrentSurfaceTexture::Suboptimal(_) | CurrentSurfaceTexture::Outdated | CurrentSurfaceTexture::Lost | CurrentSurfaceTexture::Validation => {
                match self.surface.get_current_texture() {
                    CurrentSurfaceTexture::Success(o) => o,
                    c => panic_error(c)
                }
            },
            CurrentSurfaceTexture::Timeout |
            CurrentSurfaceTexture::Occluded => {
                panic_error(CurrentSurfaceTexture::Occluded)
            },
        };

        let view = output
            .texture
            .create_view(&TextureViewDescriptor::default());
        (output, view)
    }
}
