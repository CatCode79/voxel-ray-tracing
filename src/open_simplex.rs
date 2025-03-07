//!
//! 2014 OpenSimplex Noise in Java.
//! by Kurt Spencer
//!
//! Ported to Rust by Mason Feurer (Excluding 4D) (Added `NoiseMap` and `MultiNoiseMap`).
//! I have no idea how this thing works, just translated the control flow.
//!
use glam::Vec2;

#[derive(Clone)]
pub(crate) struct NoiseMap {
    // OpenSimplexNoise is a pretty big struct, better to not store it on the stack
    noise: Box<OpenSimplexNoise>,
    scale: f64,
    freq: f64,
}

impl NoiseMap {
    pub(crate) fn new(seed: i64, freq: f64, scale: f64) -> Self {
        Self {
            noise: Box::new(OpenSimplexNoise::new(seed)),
            freq,
            scale,
        }
    }
    pub(crate) fn get(&self, pos: Vec2) -> f32 {
        let val = self
            .noise
            .eval2d(pos.x as f64 * self.freq, pos.y as f64 * self.freq);
        (((val + 1.0) * 0.5) * self.scale) as f32
    }
}

const STRETCH_CONSTANT_2D: f64 = -0.211324865405187; // (1/(2+1).sqrt()-1)/2;
const SQUISH_CONSTANT_2D: f64 = 0.366025403784439; // ((2+1).sqrt()-1)/2;

const PSIZE: usize = 2048;
const PMASK: usize = 2047;

#[derive(Clone)]
pub(crate) struct OpenSimplexNoise {
    perm: [usize; PSIZE],
    perm_grad2: [Grad2; PSIZE],
}

impl OpenSimplexNoise {
    pub(crate) fn new(mut seed: i64) -> Self {
        let mut perm = [0; PSIZE];
        let mut perm_grad2 = [Grad2::ZERO; PSIZE];
        let mut source = [0; PSIZE];
        for i in 0..PSIZE {
            source[i] = i;
        }
        #[allow(unsafe_code)]
        for i in (0..PSIZE).rev() {
            seed = seed
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let r = ((seed + 31) % (i as i64 + 1)) as isize;
            let r = if r < 0 { r + i as isize + 1 } else { r } as usize;
            perm[i] = source[r];
            perm_grad2[i] = unsafe { GRADIENTS_2D[perm[i]] };
            source[r] = source[i];
        }
        Self { perm, perm_grad2 }
    }

    /// 2D OpenSimplex Noise
    pub(crate) fn eval2d(&self, x: f64, y: f64) -> f64 {
        // Place input coordinates onto grid.
        let stretch_offset = (x + y) * STRETCH_CONSTANT_2D;
        let xs = x + stretch_offset;
        let ys = y + stretch_offset;

        // Floor to get grid coordinates of rhombus (stretched square) super-cell origin.
        let mut xsb: i32 = xs.floor() as i32;
        let mut ysb: i32 = ys.floor() as i32;

        // Compute grid coordinates relative to rhombus origin.
        let xins: f64 = xs - xsb as f64;
        let yins: f64 = ys - ysb as f64;

        // Sum those together to get a value that determines which region we're in.
        let in_sum: f64 = xins + yins;

        // Positions relative to origin point.
        let squish_offset_ins: f64 = in_sum * SQUISH_CONSTANT_2D;
        let mut dx0: f64 = xins + squish_offset_ins;
        let mut dy0: f64 = yins + squish_offset_ins;

        // We'll be defining these inside the next block and using them afterwards.
        let dx_ext: f64;
        let dy_ext: f64;
        let xsv_ext: i32;
        let ysv_ext: i32;

        let mut value: f64 = 0.0;

        // Contribution (1,0)
        let dx1: f64 = dx0 - 1.0 - SQUISH_CONSTANT_2D;
        let dy1: f64 = dy0 - 0.0 - SQUISH_CONSTANT_2D;
        let mut attn1: f64 = 2.0 - dx1 * dx1 - dy1 * dy1;
        if attn1 > 0.0 {
            attn1 *= attn1;
            value += attn1 * attn1 * self.extrapolate2d(xsb + 1, ysb + 0, dx1, dy1);
        }

        // Contribution (0,1)
        let dx2: f64 = dx0 - 0.0 - SQUISH_CONSTANT_2D;
        let dy2: f64 = dy0 - 1.0 - SQUISH_CONSTANT_2D;
        let mut attn2: f64 = 2.0 - dx2 * dx2 - dy2 * dy2;
        if attn2 > 0.0 {
            attn2 *= attn2;
            value += attn2 * attn2 * self.extrapolate2d(xsb + 0, ysb + 1, dx2, dy2);
        }

        if in_sum <= 1.0 {
            // We're inside the triangle (2-Simplex) at (0,0)
            let zins: f64 = 1.0 - in_sum;
            if zins > xins || zins > yins {
                // (0,0) is one of the closest two triangular vertices
                if xins > yins {
                    xsv_ext = xsb + 1;
                    ysv_ext = ysb - 1;
                    dx_ext = dx0 - 1.0;
                    dy_ext = dy0 + 1.0;
                } else {
                    xsv_ext = xsb - 1;
                    ysv_ext = ysb + 1;
                    dx_ext = dx0 + 1.0;
                    dy_ext = dy0 - 1.0;
                }
            } else {
                // (1,0) and (0,1) are the closest two vertices.
                xsv_ext = xsb + 1;
                ysv_ext = ysb + 1;
                dx_ext = dx0 - 1.0 - 2.0 * SQUISH_CONSTANT_2D;
                dy_ext = dy0 - 1.0 - 2.0 * SQUISH_CONSTANT_2D;
            }
        } else {
            // We're inside the triangle (2-Simplex) at (1,1)
            let zins: f64 = 2.0 - in_sum;
            if zins < xins || zins < yins {
                // (0,0) is one of the closest two triangular vertices
                if xins > yins {
                    xsv_ext = xsb + 2;
                    ysv_ext = ysb + 0;
                    dx_ext = dx0 - 2.0 - 2.0 * SQUISH_CONSTANT_2D;
                    dy_ext = dy0 + 0.0 - 2.0 * SQUISH_CONSTANT_2D;
                } else {
                    xsv_ext = xsb + 0;
                    ysv_ext = ysb + 2;
                    dx_ext = dx0 + 0.0 - 2.0 * SQUISH_CONSTANT_2D;
                    dy_ext = dy0 - 2.0 - 2.0 * SQUISH_CONSTANT_2D;
                }
            } else {
                // (1,0) and (0,1) are the closest two vertices.
                dx_ext = dx0;
                dy_ext = dy0;
                xsv_ext = xsb;
                ysv_ext = ysb;
            }
            xsb += 1;
            ysb += 1;
            dx0 = dx0 - 1.0 - 2.0 * SQUISH_CONSTANT_2D;
            dy0 = dy0 - 1.0 - 2.0 * SQUISH_CONSTANT_2D;
        }

        // Contribution (0,0) or (1,1)
        let mut attn0: f64 = 2.0 - dx0 * dx0 - dy0 * dy0;
        if attn0 > 0.0 {
            attn0 *= attn0;
            value += attn0 * attn0 * self.extrapolate2d(xsb, ysb, dx0, dy0);
        }

        // Extra Vertex
        let mut attn_ext: f64 = 2.0 - dx_ext * dx_ext - dy_ext * dy_ext;
        if attn_ext > 0.0 {
            attn_ext *= attn_ext;
            value += attn_ext * attn_ext * self.extrapolate2d(xsv_ext, ysv_ext, dx_ext, dy_ext);
        }
        value
    }

    fn extrapolate2d(&self, xsb: i32, ysb: i32, dx: f64, dy: f64) -> f64 {
        let grad = &self.perm_grad2[self.perm[xsb as usize & PMASK] ^ (ysb as usize & PMASK)];
        grad.dx * dx + grad.dy * dy
    }
}

#[derive(Clone, Copy)]
struct Grad2 {
    dx: f64,
    dy: f64,
}
impl Grad2 {
    const ZERO: Self = Grad2 { dx: 0.0, dy: 0.0 };
}
#[derive(Clone, Copy)]
struct Grad3 {
    dx: f64,
    dy: f64,
    dz: f64,
}
impl Grad3 {
    const ZERO: Self = Grad3 {
        dx: 0.0,
        dy: 0.0,
        dz: 0.0,
    };
}

const N2: f64 = 7.69084574549313;
const N3: f64 = 26.92263139946168;

static mut GRADIENTS_2D: [Grad2; PSIZE] = [Grad2::ZERO; PSIZE];
static mut GRADIENTS_3D: [Grad3; PSIZE] = [Grad3::ZERO; PSIZE];

pub(crate) fn init_gradients() {
    let mut grad2 = [
        Grad2 {
            dx: 0.130526192220052,
            dy: 0.99144486137381,
        },
        Grad2 {
            dx: 0.38268343236509,
            dy: 0.923879532511287,
        },
        Grad2 {
            dx: 0.608761429008721,
            dy: 0.793353340291235,
        },
        Grad2 {
            dx: 0.793353340291235,
            dy: 0.608761429008721,
        },
        Grad2 {
            dx: 0.923879532511287,
            dy: 0.38268343236509,
        },
        Grad2 {
            dx: 0.99144486137381,
            dy: 0.130526192220051,
        },
        Grad2 {
            dx: 0.99144486137381,
            dy: -0.130526192220051,
        },
        Grad2 {
            dx: 0.923879532511287,
            dy: -0.38268343236509,
        },
        Grad2 {
            dx: 0.793353340291235,
            dy: -0.60876142900872,
        },
        Grad2 {
            dx: 0.608761429008721,
            dy: -0.793353340291235,
        },
        Grad2 {
            dx: 0.38268343236509,
            dy: -0.923879532511287,
        },
        Grad2 {
            dx: 0.130526192220052,
            dy: -0.99144486137381,
        },
        Grad2 {
            dx: -0.130526192220052,
            dy: -0.99144486137381,
        },
        Grad2 {
            dx: -0.38268343236509,
            dy: -0.923879532511287,
        },
        Grad2 {
            dx: -0.608761429008721,
            dy: -0.793353340291235,
        },
        Grad2 {
            dx: -0.793353340291235,
            dy: -0.608761429008721,
        },
        Grad2 {
            dx: -0.923879532511287,
            dy: -0.38268343236509,
        },
        Grad2 {
            dx: -0.99144486137381,
            dy: -0.130526192220052,
        },
        Grad2 {
            dx: -0.99144486137381,
            dy: 0.130526192220051,
        },
        Grad2 {
            dx: -0.923879532511287,
            dy: 0.38268343236509,
        },
        Grad2 {
            dx: -0.793353340291235,
            dy: 0.608761429008721,
        },
        Grad2 {
            dx: -0.608761429008721,
            dy: 0.793353340291235,
        },
        Grad2 {
            dx: -0.38268343236509,
            dy: 0.923879532511287,
        },
        Grad2 {
            dx: -0.130526192220052,
            dy: 0.99144486137381,
        },
    ];
    for i in 0..grad2.len() {
        grad2[i].dx /= N2;
        grad2[i].dy /= N2;
    }
    for i in 0..PSIZE {
        #[allow(unsafe_code)]
        unsafe {
            GRADIENTS_2D[i] = grad2[i % grad2.len()];
        }
    }

    let mut grad3 = [
        Grad3 {
            dx: -1.4082482904633333,
            dy: -1.4082482904633333,
            dz: -2.6329931618533333,
        },
        Grad3 {
            dx: -0.07491495712999985,
            dy: -0.07491495712999985,
            dz: -3.29965982852,
        },
        Grad3 {
            dx: 0.24732126143473554,
            dy: -1.6667938651159684,
            dz: -2.838945207362466,
        },
        Grad3 {
            dx: -1.6667938651159684,
            dy: 0.24732126143473554,
            dz: -2.838945207362466,
        },
        Grad3 {
            dx: -1.4082482904633333,
            dy: -2.6329931618533333,
            dz: -1.4082482904633333,
        },
        Grad3 {
            dx: -0.07491495712999985,
            dy: -3.29965982852,
            dz: -0.07491495712999985,
        },
        Grad3 {
            dx: -1.6667938651159684,
            dy: -2.838945207362466,
            dz: 0.24732126143473554,
        },
        Grad3 {
            dx: 0.24732126143473554,
            dy: -2.838945207362466,
            dz: -1.6667938651159684,
        },
        Grad3 {
            dx: 1.5580782047233335,
            dy: 0.33333333333333337,
            dz: -2.8914115380566665,
        },
        Grad3 {
            dx: 2.8914115380566665,
            dy: -0.33333333333333337,
            dz: -1.5580782047233335,
        },
        Grad3 {
            dx: 1.8101897177633992,
            dy: -1.2760767510338025,
            dz: -2.4482280932803,
        },
        Grad3 {
            dx: 2.4482280932803,
            dy: 1.2760767510338025,
            dz: -1.8101897177633992,
        },
        Grad3 {
            dx: 1.5580782047233335,
            dy: -2.8914115380566665,
            dz: 0.33333333333333337,
        },
        Grad3 {
            dx: 2.8914115380566665,
            dy: -1.5580782047233335,
            dz: -0.33333333333333337,
        },
        Grad3 {
            dx: 2.4482280932803,
            dy: -1.8101897177633992,
            dz: 1.2760767510338025,
        },
        Grad3 {
            dx: 1.8101897177633992,
            dy: -2.4482280932803,
            dz: -1.2760767510338025,
        },
        Grad3 {
            dx: -2.6329931618533333,
            dy: -1.4082482904633333,
            dz: -1.4082482904633333,
        },
        Grad3 {
            dx: -3.29965982852,
            dy: -0.07491495712999985,
            dz: -0.07491495712999985,
        },
        Grad3 {
            dx: -2.838945207362466,
            dy: 0.24732126143473554,
            dz: -1.6667938651159684,
        },
        Grad3 {
            dx: -2.838945207362466,
            dy: -1.6667938651159684,
            dz: 0.24732126143473554,
        },
        Grad3 {
            dx: 0.33333333333333337,
            dy: 1.5580782047233335,
            dz: -2.8914115380566665,
        },
        Grad3 {
            dx: -0.33333333333333337,
            dy: 2.8914115380566665,
            dz: -1.5580782047233335,
        },
        Grad3 {
            dx: 1.2760767510338025,
            dy: 2.4482280932803,
            dz: -1.8101897177633992,
        },
        Grad3 {
            dx: -1.2760767510338025,
            dy: 1.8101897177633992,
            dz: -2.4482280932803,
        },
        Grad3 {
            dx: 0.33333333333333337,
            dy: -2.8914115380566665,
            dz: 1.5580782047233335,
        },
        Grad3 {
            dx: -0.33333333333333337,
            dy: -1.5580782047233335,
            dz: 2.8914115380566665,
        },
        Grad3 {
            dx: -1.2760767510338025,
            dy: -2.4482280932803,
            dz: 1.8101897177633992,
        },
        Grad3 {
            dx: 1.2760767510338025,
            dy: -1.8101897177633992,
            dz: 2.4482280932803,
        },
        Grad3 {
            dx: 3.29965982852,
            dy: 0.07491495712999985,
            dz: 0.07491495712999985,
        },
        Grad3 {
            dx: 2.6329931618533333,
            dy: 1.4082482904633333,
            dz: 1.4082482904633333,
        },
        Grad3 {
            dx: 2.838945207362466,
            dy: -0.24732126143473554,
            dz: 1.6667938651159684,
        },
        Grad3 {
            dx: 2.838945207362466,
            dy: 1.6667938651159684,
            dz: -0.24732126143473554,
        },
        Grad3 {
            dx: -2.8914115380566665,
            dy: 1.5580782047233335,
            dz: 0.33333333333333337,
        },
        Grad3 {
            dx: -1.5580782047233335,
            dy: 2.8914115380566665,
            dz: -0.33333333333333337,
        },
        Grad3 {
            dx: -2.4482280932803,
            dy: 1.8101897177633992,
            dz: -1.2760767510338025,
        },
        Grad3 {
            dx: -1.8101897177633992,
            dy: 2.4482280932803,
            dz: 1.2760767510338025,
        },
        Grad3 {
            dx: -2.8914115380566665,
            dy: 0.33333333333333337,
            dz: 1.5580782047233335,
        },
        Grad3 {
            dx: -1.5580782047233335,
            dy: -0.33333333333333337,
            dz: 2.8914115380566665,
        },
        Grad3 {
            dx: -1.8101897177633992,
            dy: 1.2760767510338025,
            dz: 2.4482280932803,
        },
        Grad3 {
            dx: -2.4482280932803,
            dy: -1.2760767510338025,
            dz: 1.8101897177633992,
        },
        Grad3 {
            dx: 0.07491495712999985,
            dy: 3.29965982852,
            dz: 0.07491495712999985,
        },
        Grad3 {
            dx: 1.4082482904633333,
            dy: 2.6329931618533333,
            dz: 1.4082482904633333,
        },
        Grad3 {
            dx: 1.6667938651159684,
            dy: 2.838945207362466,
            dz: -0.24732126143473554,
        },
        Grad3 {
            dx: -0.24732126143473554,
            dy: 2.838945207362466,
            dz: 1.6667938651159684,
        },
        Grad3 {
            dx: 0.07491495712999985,
            dy: 0.07491495712999985,
            dz: 3.29965982852,
        },
        Grad3 {
            dx: 1.4082482904633333,
            dy: 1.4082482904633333,
            dz: 2.6329931618533333,
        },
        Grad3 {
            dx: -0.24732126143473554,
            dy: 1.6667938651159684,
            dz: 2.838945207362466,
        },
        Grad3 {
            dx: 1.6667938651159684,
            dy: -0.24732126143473554,
            dz: 2.838945207362466,
        },
    ];
    for i in 0..grad3.len() {
        grad3[i].dx /= N3;
        grad3[i].dy /= N3;
        grad3[i].dz /= N3;
    }
    for i in 0..PSIZE {
        #[allow(unsafe_code)]
        unsafe {
            GRADIENTS_3D[i] = grad3[i % grad3.len()];
        }
    }
}
