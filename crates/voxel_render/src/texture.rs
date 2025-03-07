//= IMPORTS ==================================================================

use glam::U16Vec2;
use wgpu::{
    AddressMode, Device, Extent3d, FilterMode, Sampler, SamplerDescriptor, TextureDescriptor,
    TextureDimension, TextureFormat, TextureUsages, TextureView, TextureViewDescriptor,
};

//= CONSTANTS ================================================================

pub(crate) const RESULT_TEX_FORMAT: TextureFormat = TextureFormat::Rgba8Unorm;
pub(crate) const RESULT_TEX_USAGES: TextureUsages = TextureUsages::COPY_DST
    .union(TextureUsages::COPY_SRC)
    .union(TextureUsages::STORAGE_BINDING)
    .union(TextureUsages::TEXTURE_BINDING);

//= TEXTURE ==================================================================

pub(crate) struct TextureHandler {
    pub(crate) handle: wgpu::Texture,
    pub(crate) sampler: Sampler,
    pub(crate) view: TextureView,
}

impl TextureHandler {
    pub(crate) fn new(
        device: &Device,
        size: U16Vec2,
        format: TextureFormat,
        usage: TextureUsages,
    ) -> Self {
        let handle = device.create_texture(&TextureDescriptor {
            label: None,
            size: Extent3d {
                width: size.x as u32,
                height: size.y as u32,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format,
            view_formats: &[],
            usage,
        });
        let view = handle.create_view(&TextureViewDescriptor::default());

        let sampler = device.create_sampler(&SamplerDescriptor {
            label: None,
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Nearest,
            min_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Nearest,
            lod_min_clamp: 1.0,
            lod_max_clamp: 1.0,
            compare: None,
            anisotropy_clamp: 1,
            border_color: None,
        });
        Self {
            handle,
            view,
            sampler,
        }
    }

    pub(crate) fn size(&self) -> U16Vec2 {
        let size = self.handle.size();
        U16Vec2::new(size.width as u16, size.height as u16)
    }
}
