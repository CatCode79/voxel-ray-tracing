//= IMPORTS ==================================================================

use crate::supported_backends;

use pollster::FutureExt;
use wgpu::{rwh, Adapter, CompositeAlphaMode, Device, DeviceDescriptor, ExperimentalFeatures, Features, Instance, InstanceDescriptor, InstanceFlags, Limits, MemoryHints, PowerPreference, PresentMode, Queue, RequestAdapterOptions, Surface, SurfaceConfiguration, SurfaceTargetUnsafe, TextureUsages, Trace};

//= ADAPTER ==================================================================

pub fn request_adapter(
    instance: &Instance,
    surface: &Surface<'static>,
) -> Result<Adapter, String> {
    log_possible_adapters(supported_backends(), instance);

    let result = async {
        instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                compatible_surface: Some(surface),
                force_fallback_adapter: false,
            })
            .await
    }
    .block_on();

    let adapter = result.map_err(|e| format!("{e:?}({e})."))?;
    log_picked_adapter(&adapter);
    Ok(adapter)
}

/// Log all the adapters' info.
fn log_possible_adapters(backends: wgpu::Backends, wgpu_instance: &Instance) {
    for (i, adapter) in wgpu_instance
        .enumerate_adapters(backends)
        .block_on()
        .iter()
        .enumerate()
    {
        log::debug!("Possible Adapter #{}: {}", i, get_adapter_info(adapter));
    }
}

/// Log the picked adapter info.
fn log_picked_adapter(adapter: &Adapter) {
    log::info!("Picked Adapter: {}", get_adapter_info(adapter));
    log::debug!("Its Features: {:?}", adapter.features());
}

/// Return an adapter info pretty printed.
fn get_adapter_info(adapter: &Adapter) -> String {
    format!("{:?}", adapter.get_info())
        .replace("AdapterInfo { name: ", "")
        .replace(" }", "")
}

//= DEVICE AND QUEUE =========================================================

pub fn request_device(
    adapter: &Adapter,
    max_buffer_sizes: u32,
) -> Result<(Device, Queue), String> {
    let dq = async {
        adapter
            .request_device(
                &DeviceDescriptor {
                    required_features: Features::default(),
                    required_limits: Limits {
                        max_storage_buffer_binding_size: max_buffer_sizes,
                        max_buffer_size: u64::from(max_buffer_sizes),
                        ..Default::default()
                    },
                    label: None,
                    memory_hints: MemoryHints::Performance,
                    trace: Trace::default(),
                    experimental_features: ExperimentalFeatures::default(),
                },
            )
            .await
    }
    .block_on();

    let Ok(dq) = dq else {
        return Err(format!("{:?}", dq.err()));
    };

    Ok(dq)
}

//= GPU INSTANCE =============================================================

pub fn create_instance() -> Instance {
    let flags = if cfg!(debug_assertions) {
        InstanceFlags::default()
    } else {
        let mut f = InstanceFlags::empty();
        f.set(InstanceFlags::DISCARD_HAL_LABELS, true);
        f
    };

    let desc = InstanceDescriptor {
        backends: supported_backends(),
        flags,
        ..Default::default()
    };
    Instance::new(&desc)
}

//= SURFACE ==================================================================

pub fn create_surface(
    raw_display_handle: Result<rwh::RawDisplayHandle, rwh::HandleError>,
    raw_window_handle: Result<rwh::RawWindowHandle, rwh::HandleError>,
    instance: &Instance,
) -> Result<Surface<'static>, String> {
    if raw_display_handle.is_err() {
        return Err("Raw display handle error on surface creation".to_string());
    }
    if raw_window_handle.is_err() {
        return Err("Raw window handle error on surface creation".to_string());
    }

    let surface_target = SurfaceTargetUnsafe::RawHandle {
        raw_display_handle: raw_display_handle.unwrap(),
        raw_window_handle: raw_window_handle.unwrap(),
    };
    let surface = match unsafe { instance.create_surface_unsafe(surface_target) } {
        Ok(s) => s,
        Err(e) => return Err(e.to_string()),
    };

    Ok(surface)
}

pub fn create_surface_config(
    surface: &Surface<'static>,
    adapter: &Adapter,
    width: u16,
    height: u16,
) -> Result<SurfaceConfiguration, String> {
    if width == 0 {
        return Err("Impossible to create a surface configuration with zero width".to_string());
    }
    if height == 0 {
        return Err("Impossible to create a surface configuration with zero height".to_string());
    }

    let texture_formats = surface.get_capabilities(adapter).formats;
    let Some(texture_format) = texture_formats.first() else {
        return Err("A valid surface texture format isn't supported by this adapter.".to_string());
    };

    Ok(SurfaceConfiguration {
        usage: TextureUsages::RENDER_ATTACHMENT,
        format: *texture_format,
        width: u32::from(width),
        height: u32::from(height),
        desired_maximum_frame_latency: 2,
        present_mode: PresentMode::Fifo,
        alpha_mode: CompositeAlphaMode::Auto,
        view_formats: vec![],
    })
}
