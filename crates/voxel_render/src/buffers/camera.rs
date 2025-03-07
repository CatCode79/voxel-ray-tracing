//= IMPORTS ==================================================================

use glam::{Mat4, Vec2, Vec3};

//= CAMERA BUFFER ============================================================

#[derive(Clone, Copy, Default)]
#[repr(C)]
pub struct Camera {
    pub pos: Vec3,
    pub _padding0: u32,
    pub inv_view_mat: Mat4,
    pub inv_proj_mat: Mat4,
    pub proj_size: Vec2,
    pub _padding1: [u32; 2],
}
