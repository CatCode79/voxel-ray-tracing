//= IMPORTS ========================================================================================

use glam::Vec4;

// = COLOR =========================================================================================

#[derive(Clone, Copy, Debug)]
pub struct Color(Vec4);

impl Color {
    pub const N_CHANNELS: u16 = 4;

    pub const GAMMA: f32 = 2.2;

    pub const MIN_VALUE: u8 = u8::MIN;
    pub const MAX_VALUE: u8 = u8::MAX;

    pub const MIN_OPAQUE: u8 = u8::MIN;
    pub const MAX_OPAQUE: u8 = u8::MAX;

    pub const MIN_TRANSPARENT: u8 = Self::MAX_OPAQUE;
    pub const MAX_TRANSPARENT: u8 = Self::MIN_OPAQUE;

    #[must_use]
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self(Vec4::new(r, g, b, a))
    }

    #[must_use]
    pub fn new_bytes(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self(Vec4::new(
            f32::from(r) / 255.0,
            f32::from(g) / 255.0,
            f32::from(b) / 255.0,
            f32::from(a) / 255.0,
        ))
    }

    #[must_use]
    pub const fn from_rgba(d: [f32; 4]) -> Self {
        Self::new(d[0], d[1], d[2], d[3])
    }

    #[must_use]
    pub const fn from_rgb_alpha(d: [f32; 3], alpha: f32) -> Self {
        Self::new(d[0], d[1], d[2], alpha)
    }

    #[must_use]
    pub fn r(&self) -> f32 {
        self.0.x
    }

    #[must_use]
    pub fn g(&self) -> f32 {
        self.0.y
    }

    #[must_use]
    pub fn b(&self) -> f32 {
        self.0.z
    }

    #[must_use]
    pub fn a(&self) -> f32 {
        self.0.w
    }

    #[must_use]
    pub fn r_to_byte(&self) -> u8 {
        f32::round(self.0.x * 255.0) as u8
    }

    #[must_use]
    pub fn g_to_byte(&self) -> u8 {
        f32::round(self.0.y * 255.0) as u8
    }

    #[must_use]
    pub fn b_to_byte(&self) -> u8 {
        f32::round(self.0.z * 255.0) as u8
    }

    #[must_use]
    pub fn a_to_byte(&self) -> u8 {
        f32::round(self.0.w * 255.0) as u8
    }

    #[must_use]
    pub fn into_vec4(self) -> [f32; 4] {
        [self.r(), self.g(), self.b(), self.a()]
    }

    #[must_use]
    pub fn into_vec4_gamma(self) -> [f32; 4] {
        [
            self.r().powf(Self::GAMMA),
            self.g().powf(Self::GAMMA),
            self.b().powf(Self::GAMMA),
            self.a().powf(Self::GAMMA),
        ]
    }
}

impl Default for Color {
    fn default() -> Self {
        Self::new(0.0, 0.0, 0.0, 1.0) // Black
    }
}

impl From<Color> for Vec4 {
    fn from(color: Color) -> Self {
        color.0
    }
}

impl From<Color> for wgt::Color {
    fn from(c: Color) -> Self {
        Self {
            r: f64::from(c.r()),
            g: f64::from(c.g()),
            b: f64::from(c.b()),
            a: f64::from(c.a()),
        }
    }
}
