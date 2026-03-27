//= MODS ===========================================================================================

extern crate core;

mod buffers;
mod gpu;
mod passes;
mod renderer;
mod texture;

//= RE-EXPORTS =====================================================================================

pub use buffers::*;
pub use renderer::*;

//= BACKENDS =======================================================================================

/// The backends are in order of support, the greater the first.
const fn supported_backends() -> wgpu::Backends {
    #[cfg(target_os = "windows")]
    return wgpu::Backends::VULKAN /*| wgpu::Backends::DX12*/;

    #[cfg(target_os = "linux")]
    return wgpu::Backends::VULKAN;

    #[cfg(target_os = "macos")]
    return wgpu::Backends::METAL;
}
