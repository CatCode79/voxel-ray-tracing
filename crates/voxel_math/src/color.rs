//= IMPORTS ==================================================================

use glam::Vec4;

// = COLOR ===================================================================

#[derive(Clone, Copy, Debug)]
pub struct Color(Vec4);

impl Color {
    pub const N_COLORS: u16 = 4;

    pub const GAMMA: f32 = 2.2;

    pub const MIN_VALUE: u8 = 0;
    pub const MAX_VALUE: u8 = u8::MAX;

    pub const MIN_OPAQUE: u8 = u8::MIN;
    pub const MAX_OPAQUE: u8 = u8::MAX;

    pub const MIN_TRANSPARENT: u8 = Color::MAX_OPAQUE;
    pub const MAX_TRANSPARENT: u8 = Color::MIN_OPAQUE;

    #[inline]
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self(Vec4::new(r, g, b, a))
    }

    pub fn new_bytes(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self(Vec4::new(
            r as f32 / 255.0,
            g as f32 / 255.0,
            b as f32 / 255.0,
            a as f32 / 255.0,
        ))
    }

    #[inline]
    pub const fn from_rgba(d: [f32; 4]) -> Self {
        Self::new(d[0], d[1], d[2], d[3])
    }

    #[inline]
    pub const fn from_rgb_alpha(d: [f32; 3], alpha: f32) -> Self {
        Self::new(d[0], d[1], d[2], alpha)
    }

    #[inline]
    pub fn r(&self) -> f32 {
        self.0.x
    }

    #[inline]
    pub fn g(&self) -> f32 {
        self.0.y
    }

    #[inline]
    pub fn b(&self) -> f32 {
        self.0.z
    }

    #[inline]
    pub fn a(&self) -> f32 {
        self.0.w
    }

    pub fn r_to_byte(&self) -> u8 {
        f32::round(self.0.x * 255.0) as u8
    }

    pub fn g_to_byte(&self) -> u8 {
        f32::round(self.0.y * 255.0) as u8
    }

    pub fn b_to_byte(&self) -> u8 {
        f32::round(self.0.z * 255.0) as u8
    }

    pub fn a_to_byte(&self) -> u8 {
        f32::round(self.0.w * 255.0) as u8
    }

    pub fn into_vec4(self) -> [f32; 4] {
        [self.r(), self.g(), self.b(), self.a()]
    }

    pub fn into_vec4_gamma(self) -> [f32; 4] {
        [
            self.r().powf(Color::GAMMA),
            self.g().powf(Color::GAMMA),
            self.b().powf(Color::GAMMA),
            self.a().powf(Color::GAMMA),
        ]
    }

    //- Names ----------------------------------------------------------------

    pub const BLACK: Self = Self::new(0.0, 0.0, 0.0, 1.0);
    pub const BLACK_TRANSPARENT: Self = Self::new(0.0, 0.0, 0.0, 0.0);
    pub const WHITE: Self = Self::new(1.0, 1.0, 1.0, 1.0);
    pub const WHITE_TRANSPARENT: Self = Self::new(1.0, 1.0, 1.0, 0.0);
    pub const RED: Self = Self::new(1.0, 0.0, 0.0, 1.0);
    pub const GREEN: Self = Self::new(0.0, 1.0, 0.0, 1.0);
    pub const BLUE: Self = Self::new(0.0, 0.0, 1.0, 1.0);

    ///! https://en.wikipedia.org/wiki/Shades_of_brown
    pub const BROWN_BEAVER: Self = Self::new(159.0 / 255.0, 129.0 / 255.0, 112.0 / 255.0, 1.0);
    pub const BROWN_DARK: Self = Self::new(92.0 / 255.0, 64.0 / 255.0, 51.0 / 255.0, 1.0);
    pub const BROWN_PALE: Self = Self::new(152.0 / 255.0, 118.0 / 255.0, 84.0 / 255.0, 1.0);
}

impl Default for Color {
    fn default() -> Self {
        Color::BLACK
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
            r: c.r() as f64,
            g: c.g() as f64,
            b: c.b() as f64,
            a: c.a() as f64,
        }
    }
}
