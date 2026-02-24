//= MODS =====================================================================

extern crate core;

mod buffers;
mod gpu;
mod renderer;
mod shader;
mod texture;
mod passes;

//= RE-EXPORTS ===============================================================

pub use buffers::*;
pub use renderer::*;

//= BACKENDS =================================================================

/// The backends are in order of support, the greater the first.
fn supported_backends() -> wgpu::Backends {
    #[cfg(target_os = "windows")]
    return wgpu::Backends::DX12 | wgpu::Backends::VULKAN;

    #[cfg(target_os = "linux")]
    return wgpu::Backends::VULKAN;

    #[cfg(target_os = "macos")]
    return wgpu::Backends::METAL;
}
