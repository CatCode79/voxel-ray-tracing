//= MODULES ==================================================================

pub mod input;
pub mod mapping;
pub mod window;

//= IMPORTS ==================================================================

use std::ops::BitAnd;

//= UTILITY FUNCTIONS ========================================================

#[inline(always)]
pub(crate) const fn unsigned_loword(x: u32) -> u16 {
    (x & 0xFFFF) as u16
}

#[inline(always)]
pub(crate) const fn unsigned_hiword(x: u32) -> u16 {
    ((x >> 16) & 0xFFFF) as u16
}

#[inline(always)]
pub(crate) const fn signed_loword(x: u32) -> i16 {
    (x & 0xFFFF) as i16
}

#[inline(always)]
pub(crate) const fn signed_hiword(x: u32) -> i16 {
    ((x >> 16) & 0xFFFF) as i16
}

#[inline(always)]
pub(crate) const fn primarylangid(lgid: u16) -> u16 {
    lgid & 0x3FF
}

#[inline(always)]
pub fn has_flag<T>(bitset: T, flag: T) -> bool
where
    T: Copy + PartialEq + BitAnd<T, Output = T>,
{
    bitset & flag == flag
}
