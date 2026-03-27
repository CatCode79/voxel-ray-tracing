//= IMPORTS ========================================================================================

use glam::{IVec3, ivec3};

//= MACROS =========================================================================================

macro_rules! linewalker_step {
    ($self:ident; main=$m:ident, a1=$s1:ident, a2=$s2:ident) => {{
        if $self.a.$m != $self.b.$m {
            $self.a.$m += $self.step.$m;

            if $self.p1 >= 0 {
                $self.a.$s1 += $self.step.$s1;
                $self.p1 -= 2 * $self.dist.$m;
            }
            if $self.p2 >= 0 {
                $self.a.$s2 += $self.step.$s2;
                $self.p2 -= 2 * $self.dist.$m;
            }

            $self.p1 += 2 * $self.dist.$s1;
            $self.p2 += 2 * $self.dist.$s2;

            return Some($self.a);
        }
        None
    }};
}

//= LINE WALKER ====================================================================================

struct LineWalker {
    a: IVec3,
    b: IVec3,
    dist: IVec3,
    step: IVec3,
    p1: i32,
    p2: i32,
    mode: u8,
}

impl Iterator for LineWalker {
    type Item = IVec3;

    fn next(&mut self) -> Option<IVec3> {
        match self.mode {
            0 => linewalker_step!(self; main=x, a1=y, a2=z),
            1 => linewalker_step!(self; main=y, a1=x, a2=z),
            2 => linewalker_step!(self; main=z, a1=y, a2=x),
            _ => unreachable!(),
        }
    }
}

pub fn walk_line(a: IVec3, b: IVec3) -> impl Iterator<Item = IVec3> {
    let dist = (b - a).abs();
    let step = ivec3(
        i32::from(b.x > a.x) * 2 - 1,
        i32::from(b.y > a.y) * 2 - 1,
        i32::from(b.z > a.z) * 2 - 1,
    );

    let (mode, p1, p2) = if dist.x >= dist.y && dist.x >= dist.z {
        (0, 2 * dist.y - dist.x, 2 * dist.z - dist.x)
    } else if dist.y >= dist.x && dist.y >= dist.z {
        (1, 2 * dist.x - dist.y, 2 * dist.z - dist.y)
    } else {
        (2, 2 * dist.y - dist.z, 2 * dist.x - dist.z)
    };

    let walker = LineWalker {
        a,
        b,
        dist,
        step,
        p1,
        p2,
        mode,
    };
    std::iter::once(a).chain(walker)
}
