//= MODS =====================================================================

mod buffers;
pub(crate) mod gpu;
mod renderer;
mod shader;
pub(crate) mod texture;

//= RE-EXPORTS ===============================================================

pub use buffers::*;
pub use renderer::*;
pub use shader::*;

//= BACKENDS =================================================================

/// The backends are in order of support, the greater the first.
fn supported_backends() -> wgpu::Backends {
    #[cfg(target_os = "windows")]
    return /*wgpu::Backends::DX12 |*/ wgpu::Backends::VULKAN;

    #[cfg(target_os = "linux")]
    return wgpu::Backends::VULKAN;

    #[cfg(target_os = "macos")]
    return wgpu::Backends::METAL;
}
