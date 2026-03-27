//= IMPORTS ============================================================================================================

use glam::Vec3;

//= CONSTANTS ==========================================================================================================

const EPSILON: f32 = 0.00001;

//= MACROS =============================================================================================================

macro_rules! clip_axis_body {
    ($self:ident, $c:ident, $a:ident; move=$axis:ident, check=$b:ident, $c_axis:ident) => {{
        if $c.to.$b <= $self.from.$b || $c.from.$b >= $self.to.$b {
            return $a;
        }
        if $c.to.$c_axis <= $self.from.$c_axis || $c.from.$c_axis >= $self.to.$c_axis {
            return $a;
        }

        if $a > 0.0 && $c.to.$axis <= $self.from.$axis {
            let max = $self.from.$axis - $c.to.$axis - EPSILON;
            if max < $a {
                $a = max;
            }
        }
        if $a < 0.0 && $c.from.$axis >= $self.to.$axis {
            let max = $self.to.$axis - $c.from.$axis + EPSILON;
            if max > $a {
                $a = max;
            }
        }

        $a
    }};
}

//= AABB ===============================================================================================================

#[derive(Clone, Copy)]
pub struct Aabb {
    pub from: Vec3,
    pub to: Vec3,
}

impl Aabb {
    pub const UNIT: Self = Self::new(Vec3::ZERO, Vec3::ONE);

    #[must_use]
    pub const fn new(from: Vec3, to: Vec3) -> Self {
        Self { from, to }
    }

    #[must_use]
    pub fn expand(&self, a: Vec3) -> Self {
        let mut from = self.from;
        let mut to = self.to;

        if a.x < 0.0 {
            from.x += a.x;
        }
        if a.x > 0.0 {
            to.x += a.x;
        }

        if a.y < 0.0 {
            from.y += a.y;
        }
        if a.y > 0.0 {
            to.y += a.y;
        }

        if a.z < 0.0 {
            from.z += a.z;
        }
        if a.z > 0.0 {
            to.z += a.z;
        }

        Self::new(from, to)
    }

    #[must_use]
    pub fn grow(&self, a: Vec3) -> Self {
        Self::new(self.from - a, self.to + a)
    }

    #[must_use]
    pub fn clip_x_collide(&self, c: &Self, mut a: f32) -> f32 {
        clip_axis_body!(self, c, a; move=x, check=y, z)
    }

    #[must_use]
    pub fn clip_y_collide(&self, c: &Self, mut a: f32) -> f32 {
        clip_axis_body!(self, c, a; move=y, check=x, z)
    }

    #[must_use]
    pub fn clip_z_collide(&self, c: &Self, mut a: f32) -> f32 {
        clip_axis_body!(self, c, a; move=z, check=x, y)
    }

    #[must_use]
    pub fn intersects(&self, c: &Self) -> bool {
        (c.to.x > self.from.x && c.from.x < self.to.x)
            && (c.to.y > self.from.y && c.from.y < self.to.y)
            && (c.to.z > self.from.z && c.from.z < self.to.z)
    }

    pub fn translate(&mut self, a: Vec3) {
        self.from += a;
        self.to += a;
    }
}
