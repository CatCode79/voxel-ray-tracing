//= SETTINGS BUFFER ==========================================================

#[derive(Clone, Copy, Default)]
#[repr(C)]
pub struct Settings {
    pub max_ray_bounces: u32,
    pub samples_per_pixel: u32,
    pub sun_intensity: f32,
    pub _padding0: u32,
    pub sky_color: [f32; 3],
    pub _padding1: u32,
    pub sun_pos: [f32; 3],
    pub _padding2: u32,
}
