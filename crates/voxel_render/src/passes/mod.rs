//= MODS ===========================================================================================

mod denoiser;
mod path_tracer;
mod ray_tracer;
mod screen;

//= RE-EXPORTS =====================================================================================

pub(crate) use denoiser::*;
pub(crate) use path_tracer::*;
pub(crate) use ray_tracer::*;
pub(crate) use screen::*;

//= IMPORTS ========================================================================================

use wgpu::{BindingType, BufferBindingType};

//= CONSTANTS ================================================================

static RAY_TRACER_SRC: &str = include_str!("../shaders/ray_tracer.wgsl");
static PATH_TRACER_SRC: &str = include_str!("../shaders/path_tracer.wgsl");
static DENOISER_SRC: &str = include_str!("../shaders/denoiser.wgsl");
static SCREEN_SHADER_SRC: &str = include_str!("../shaders/screen.wgsl");

//= BINDING TYPES ============================================================

const fn uniform_binding_type() -> BindingType {
    BindingType::Buffer {
        ty: BufferBindingType::Uniform,
        has_dynamic_offset: false,
        min_binding_size: None,
    }
}

const fn storage_binding_type(read_only: bool) -> BindingType {
    BindingType::Buffer {
        ty: BufferBindingType::Storage { read_only },
        has_dynamic_offset: false,
        min_binding_size: None,
    }
}

//= MACROS ===================================================================

#[macro_export]
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

#[macro_export]
macro_rules! bind_group_entries {
    ($($binding:expr=>$entry:expr),*$(,)?) => {{[
        $(BindGroupEntry {
            binding: $binding,
            resource: $entry,
        }),*
    ]}}
}
