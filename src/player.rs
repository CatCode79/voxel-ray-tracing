//= IMPORTS =======================================================================================

use crate::world::World;

use voxel_math::{
    aabb::Aabb,
    dda::{axis_rot_to_ray, cast_ray, HitResult},
};
use voxel_render::{Camera, Voxel};
use voxel_winput::{mapping::InputKind, window::Window};

use glam::{vec3, BVec3, Mat4, U16Vec2, Vec3};

//= GRAVITY FALLS =================================================================================

const GRAVITY: f32 = -0.060;

//= PLAYER ========================================================================================

#[derive(Clone)]
pub struct Player {
    pub fov: f32,

    pub flying: bool,
    pub on_ground: bool,

    pub position: Vec3,
    pub rotation: Vec3, // (in degrees)
    pub velocity: Vec3,
    pub speed: f32,
}

impl Player {
    pub fn new(position: Vec3, speed: f32) -> Self {
        Self {
            fov: 70.0,

            flying: false,
            on_ground: false,

            position,
            rotation: Vec3::ZERO,
            velocity: Vec3::ZERO,
            speed,
        }
    }

    pub fn create_aabb(&self) -> Aabb {
        const WIDTH: f32 = 0.6;
        const HEIGHT: f32 = 3.8;

        Aabb::new(
            self.position - Vec3::new(WIDTH, 0.0, WIDTH) * 0.5,
            self.position + Vec3::new(WIDTH, HEIGHT * 2.0, WIDTH) * 0.5,
        )
    }

    pub fn apply_acc(&mut self, v: Vec3) {
        self.velocity += v;
    }

    pub fn update(&mut self, window: &Window, world: &World) {
        let frame_modifier = window.get_frame_modifier() as f32;

        let dx = self.rotation.y.to_radians().sin() * self.speed * frame_modifier;
        let dz = self.rotation.y.to_radians().cos() * self.speed * frame_modifier;

        if let Some(rotation) = window.handle_cursor_movement() {
            self.rotation.x = self.rotation.x + rotation.x * frame_modifier;
            self.rotation.y = self.rotation.y - rotation.y * frame_modifier;
        }

        if self.flying {
            self.velocity.y = 0.0;
        }
        if !self.flying {
            self.apply_acc(Vec3::new(0.0, GRAVITY, 0.0));
        }
        self.velocity *= 0.96;

        let mut frame_vel = self.velocity;

        // Flying toggle
        if window.get_input_state(InputKind::Flying).is_some() {
            self.flying = !self.flying;
            if self.flying {
                self.velocity = Vec3::ZERO;
                return;
            }
        }

        if window.get_input_state(InputKind::WalkForward).is_some() {
            frame_vel.x += -dx;
            frame_vel.z += -dz;
        }
        if window.get_input_state(InputKind::WalkLeft).is_some() {
            frame_vel.x += -dz;
            frame_vel.z += dx;
        }
        if window.get_input_state(InputKind::WalkBackward).is_some() {
            frame_vel.x += dx;
            frame_vel.z += dz;
        }
        if window.get_input_state(InputKind::WalkRight).is_some() {
            frame_vel.x += dz;
            frame_vel.z += -dx;
        }
        if self.flying {
            if window.get_input_state(InputKind::Jump).is_some() {
                frame_vel.y += self.speed * frame_modifier;
            }
            if window.get_input_state(InputKind::SlowPace).is_some() {
                frame_vel.y += -self.speed * frame_modifier;
            }
        } else if window.get_input_state(InputKind::Jump).is_some() {
            self.velocity.y = 0.6;
            self.on_ground = false;
            frame_vel.y = 0.6;
        }
        self.attempt_movement(world, frame_vel);
    }

    pub fn eye_pos(&self) -> Vec3 {
        self.position + Vec3::new(0.0, 3.6, 0.0)
    }

    pub fn create_view_mat(&self) -> Mat4 {
        Mat4::from_translation(self.eye_pos())
            * Mat4::from_rotation_x(self.rotation.x.to_radians())
            * Mat4::from_rotation_y(-self.rotation.y.to_radians())
            * Mat4::from_rotation_z(self.rotation.z.to_radians())
    }
    pub fn create_inv_view_mat(&self) -> Mat4 {
        Mat4::from_rotation_x(self.rotation.x.to_radians())
            * Mat4::from_rotation_y(-self.rotation.y.to_radians())
            * Mat4::from_rotation_z(self.rotation.z.to_radians())
            * Mat4::from_translation(-self.eye_pos())
    }

    pub fn create_proj_mat(&self, aspect: f32) -> Mat4 {
        Mat4::perspective_rh(self.fov.to_radians(), aspect, 0.001, 1000.0)
    }

    pub fn create_camera(&self, proj_size: U16Vec2) -> Camera {
        let proj_size = proj_size.as_vec2();
        let inv_view_mat = self.create_view_mat();
        let inv_proj_mat = self.create_proj_mat(proj_size.x / proj_size.y).inverse();

        Camera {
            pos: self.eye_pos(),
            inv_view_mat,
            inv_proj_mat,
            proj_size,
            ..Default::default()
        }
    }

    fn attempt_movement(&mut self, world: &World, mv: Vec3) {
        if self.flying {
            self.position += mv;
            return;
        }

        struct ClippedMovement {
            result: Vec3,
            eq: BVec3,
        }

        let clip_movement = |world: &World, bbox: Aabb, mv: Vec3| -> ClippedMovement {
            let world_bboxs = world.get_collisions_w(&bbox.expand(mv));

            let mut result = mv;
            for world_bbox in &world_bboxs {
                result.y = world_bbox.clip_y_collide(&bbox, result.y);
                result.x = world_bbox.clip_x_collide(&bbox, result.x);
                result.z = world_bbox.clip_z_collide(&bbox, result.z);
            }
            ClippedMovement {
                result,
                eq: result.cmpeq(mv),
            }
        };
        let mut bbox = self.create_aabb();

        let ClippedMovement {
            result: mv_clipped,
            eq,
        } = clip_movement(world, bbox, mv);

        self.velocity *= vec3(eq.x as i32 as f32, eq.y as i32 as f32, eq.z as i32 as f32);

        if !eq.x || !eq.z {
            // if we've been stopped in the X or Z direction,
            // test if we would be able to move forward if we were higher up.
            bbox.translate(vec3(0.0, 1.1, 0.0));

            let mut up_mv_clipped = clip_movement(world, bbox, mv);
            up_mv_clipped.result.y = 0.0;

            // if you can move further in any direction when one space higher, then we should jump
            if up_mv_clipped.result.abs().cmpgt(mv_clipped.abs()).any() {
                self.position += vec3(0.0, 1.1, 0.0);
            }
        }

        self.on_ground = self.velocity.y == 0.0 && mv.y < 0.0;
        self.position += mv_clipped;
    }

    pub fn cast_ray(&self, world: &World) -> Option<HitResult> {
        cast_ray(
            self.eye_pos(),
            axis_rot_to_ray(Vec3::new(
                self.rotation.x.to_radians(),
                self.rotation.y.to_radians(),
                self.rotation.z.to_radians(),
            )),
            100.0,
            |pos| world.get_voxel(pos).map(Voxel::is_solid).unwrap_or(false),
        )
    }
}
