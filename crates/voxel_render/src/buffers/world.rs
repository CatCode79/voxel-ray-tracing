//= WORLD DATA ===============================================================

use glam::IVec3;

#[derive(Clone, Copy, Default)]
#[repr(C)]
pub struct WorldData {
    pub min: [f32; 3],
    pub size: f32,
}

impl WorldData {
    pub fn new(min: IVec3, size: u32) -> Self {
        Self {
            min: [min.x as f32, min.y as f32, min.z as f32],
            size: size as f32,
        }
    }
}
