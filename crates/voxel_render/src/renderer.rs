//= IMPORTS ==================================================================

use crate::gpu::{
    create_instance, create_surface, create_surface_config, request_adapter, request_device,
};
use crate::texture::{TextureHandler, RESULT_TEX_FORMAT, RESULT_TEX_USAGES};
use crate::{
    Buffers, Camera, DenoiserShader, FrameData, RayTracerShader, Material, Node, PathTracerShader, ScreenShader, Settings,
    WorldData,
};

use glam::U16Vec2;
use raw_window_handle as rwh;
use wgpu::{CommandBuffer, CommandEncoder, CommandEncoderDescriptor, Device, Limits, Queue, Surface, SurfaceConfiguration, SurfaceError, SurfaceTexture, TextureView, TextureViewDescriptor};

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
    output: Option<SurfaceTexture>,

    ray_tracer: RayTracerShader,
    path_tracer: PathTracerShader,
    denoiser_shader: DenoiserShader,
    screen_shader: ScreenShader,
}

impl Renderer {
    pub fn new(
        raw_display_handle: Result<rwh::RawDisplayHandle, rwh::HandleError>,
        raw_window_handle: Result<rwh::RawWindowHandle, rwh::HandleError>,
        surface_width: u16,
        surface_height: u16,
        max_nodes: u32,
        _sample_count: u8,
    ) -> Result<Self, String> {
        let instance = create_instance();

        let surface = create_surface(raw_display_handle, raw_window_handle, &instance)?;

        let adapter = request_adapter(instance, &surface)?;

        let surface_config =
            create_surface_config(&surface, &adapter, surface_width, surface_height)?;

        let (device, queue) = request_device(adapter, Renderer::max_buffer_sizes())?;

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

        let ray_tracer =
            RayTracerShader::new(&device, &normal_texture, &buffers);
        let path_tracer =
            PathTracerShader::new(&device, &result_texture, &prev_result_texture, &buffers);
        let denoiser_shader = DenoiserShader::new(&device, &result_texture, &denoised_texture);
        let screen_shader = ScreenShader::new(&device, &denoised_texture, surface_config.format);

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
            output: None,

            ray_tracer,
            path_tracer,
            screen_shader,
            denoiser_shader,
        })
    }

    pub fn max_buffer_sizes() -> u32 {
        Limits::default().max_storage_buffer_binding_size
    }

    //= ENCODER AND SUBMIT PASS ==============================================

    pub fn create_command_encoder(&self) -> CommandEncoder {
        self.device
            .create_command_encoder(&CommandEncoderDescriptor::default())
    }

    pub fn submit_once(&mut self, command_buffer: CommandBuffer, output: SurfaceTexture) {
        self.queue.submit(iter::once(command_buffer));
        self.output = Some(output);
    }

    //= BUFFERS ==============================================================

    pub fn write_camera(&self, camera: &Camera) {
        self.buffers.camera_buffer.write(&self.queue, &camera);
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
            .write_slice(&self.queue, offset, &voxel_materials);
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
            self.surface_config.width = width as u32;
            self.surface_config.height = height as u32;
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

            self.ray_tracer.recreate_bind_group(
                &self.device,
                &self.normal_texture,
                &self.buffers,
            );
            self.path_tracer.recreate_bind_group(
                &self.device,
                &self.result_texture,
                &self.prev_result_texture,
                &self.buffers,
            );
            self.denoiser_shader.recreate_bind_group(
                &self.device,
                &self.result_texture,
                &self.denoised_texture,
            );
            self.screen_shader
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

    pub fn reset_frame_counter(&mut self) {
        self.frame_data.reset();
    }

    pub fn update(&mut self, camera: Camera) -> Result<(), String> {
        let (output, view) = self.get_output().map_err(|e| e.to_string())?;
        let surface_size = self.surface_size();
        let mut encoder = self.create_command_encoder();

        {
            self.write_frame_data(&self.frame_data);
            self.frame_data.increment();
            self.write_camera(&camera);
        }

        let workgroups = surface_size / 8;
        self.ray_tracer.encode_pass(&mut encoder, workgroups);
        self.path_tracer.encode_pass(&mut encoder, workgroups);
        self.denoiser_shader.encode_pass(&mut encoder, workgroups);
        self.screen_shader.encode_pass(&mut encoder, &view);

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
                width: self.result_texture.size().x as u32,
                height: self.result_texture.size().y as u32,
                depth_or_array_layers: 1,
            },
        );

        self.submit_once(encoder.finish(), output);

        Ok(())
    }

    //= SURFACE TEXTURE ======================================================

    pub fn surface_size(&self) -> U16Vec2 {
        U16Vec2::new(
            self.surface_config.width as u16,
            self.surface_config.height as u16,
        )
    }

    pub fn get_output(&self) -> Result<(SurfaceTexture, TextureView), SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&TextureViewDescriptor::default());
        Ok((output, view))
    }

    pub fn present(&mut self) {
        if self.output.is_some() {
            self.output.take().unwrap().present();
        }
    }
}
