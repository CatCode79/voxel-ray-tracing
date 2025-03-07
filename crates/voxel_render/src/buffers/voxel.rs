//= VOXEL NAMES ==============================================================

pub static VOXEL_NAMES: &[&str] = &[
    "Air",
    "Stone",
    "Dirt",
    "Grass",
    "Snow",
    "Dead Grass",
    "Moist Grass",
    "Sand",
    "Mud",
    "Clay",
    "Fire",
    "Magma",
    "Water",
    "Oak Wood",
    "Oak Leaves",
    "Birch Wood",
    "Birch Leaves",
    "Spruce Wood",
    "Spruce Leaves",
    "Cactus",
    "Gold",
    "Mirror",
    "Bright",
];

//= VOXEL MATERIALS ==========================================================

pub static VOXEL_MATERIALS: &[Material] = &[
    Material::new_empty(),                                  // Air
    Material::new_solid([0.40, 0.40, 0.40], 1.0),           // Stone
    Material::new_solid([0.40, 0.20, 0.00], 1.0),           // Dirt
    Material::new_solid([0.011, 0.58, 0.11], 1.0),          // Grass
    Material::new_solid([1.0; 3], 0.8),                     // Snow
    Material::new_solid([0.2, 0.4, 0.2], 1.0),              // Dead Grass
    Material::new_solid([1.0, 0.0, 0.0], 1.0),              // Moist Grass
    Material::new_solid([0.99, 0.92, 0.53], 0.9),           // Sand
    Material::new_solid([0.22, 0.13, 0.02], 0.8),           // Mud
    Material::new_solid([0.35, 0.30, 0.25], 0.8),           // Clay
    Material::new_solid([1.00, 0.90, 0.20], 0.0).emit(2.0), // Fire
    Material::new_solid([0.75, 0.18, 0.01], 1.0).emit(1.0), // Magma
    Material::new_solid([0.076, 0.563, 0.563], 0.0),        // Water
    Material::new_solid([0.25, 0.10, 0.00], 1.0),           // Oak Wood
    Material::new_solid([0.23, 0.52, 0.00], 1.0),           // Oak Leaves
    Material::new_solid([1.0; 3], 1.0),                     // Birch Wood
    Material::new_solid([0.43, 0.72, 0.00], 1.0),           // Birch Leaves
    Material::new_solid([0.06, 0.04, 0.00], 1.0),           // Spruce Wood
    Material::new_solid([0.04, 0.22, 0.00], 1.0),           // Spruce Leaves
    Material::new_solid([0.0, 0.30, 0.0], 1.0),             // Cactus
    Material::new_solid([0.83, 0.68, 0.22], 0.3),           // Gold
    Material::new_solid([1.0; 3], 0.0),                     // Mirror
    Material::new_solid([1.0; 3], 1.0).emit(5.0),           // Bright
];

//= VOXEL ====================================================================

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct Voxel(pub u8);

impl Voxel {
    pub const AIR: Self = Self(0);
    pub const STONE: Self = Self(1);
    pub const DIRT: Self = Self(2);
    pub const GRASS: Self = Self(3);
    pub const SNOW: Self = Self(4);
    pub const DEAD_GRASS: Self = Self(5);
    pub const MOIST_GRASS: Self = Self(6);
    pub const SAND: Self = Self(7);
    pub const MUD: Self = Self(8);
    pub const CLAY: Self = Self(9);
    pub const FIRE: Self = Self(10);
    pub const MAGMA: Self = Self(11);
    pub const WATER: Self = Self(12);
    pub const OAK_WOOD: Self = Self(13);
    pub const OAK_LEAVES: Self = Self(14);
    pub const BIRCH_WOOD: Self = Self(15);
    pub const BIRCH_LEAVES: Self = Self(16);
    pub const SPRUCE_WOOD: Self = Self(17);
    pub const SPRUCE_LEAVES: Self = Self(18);
    pub const CACTUS: Self = Self(19);
    pub const GOLD: Self = Self(20);
    pub const MIRROR: Self = Self(21);
    pub const BRIGHT: Self = Self(22);

    #[inline(always)]
    pub fn display_name(&self) -> &'static str {
        VOXEL_NAMES[self.0 as usize]
    }

    #[inline(always)]
    pub fn is_empty(self) -> bool {
        self == Self::AIR || self == Self::WATER
    }
    #[inline(always)]
    pub fn is_solid(self) -> bool {
        match self {
            Self::AIR => false,
            Self::WATER => false,
            Self::MAGMA => false,
            Self::FIRE => false,
            Self::MUD => false,
            _ => true,
        }
    }

    #[inline(always)]
    pub fn viscosity(self) -> f32 {
        match self {
            Self::AIR => 1.0,
            Self::WATER => 0.6,
            Self::MAGMA => 0.2,
            Self::FIRE => 1.0,
            Self::MUD => 0.2,
            _ => 0.0,
        }
    }
}

//= MATERIAL =================================================================

#[derive(Clone, Copy)]
#[repr(C)]
pub struct Material {
    pub color: [f32; 3],
    pub empty: u32,
    pub scatter: f32,
    pub emission: f32,
    pub polish_bounce_chance: f32,
    pub _padding0: u32,
    pub polish_color: [f32; 3],
    pub polish_scatter: f32,
}

impl Material {
    pub const ZERO: Self = Self {
        color: [0.0; 3],
        empty: 0,
        scatter: 0.0,
        emission: 0.0,
        polish_bounce_chance: 0.0,
        _padding0: 0,
        polish_color: [0.0; 3],
        polish_scatter: 0.0,
    };

    pub const fn new_empty() -> Self {
        let mut rs = Self::ZERO;
        rs.empty = 1;
        rs
    }

    pub const fn new_solid(color: [f32; 3], scatter: f32) -> Self {
        let mut rs = Self::ZERO;
        rs.color = color;
        rs.scatter = scatter;
        rs
    }

    pub const fn polished(mut self, chance: f32, scatter: f32, color: [f32; 3]) -> Self {
        self.polish_bounce_chance = chance;
        self.polish_color = color;
        self.polish_scatter = scatter;
        self
    }

    pub const fn emit(mut self, emission: f32) -> Self {
        self.emission = emission;
        self
    }
}
