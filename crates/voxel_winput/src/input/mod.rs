//= MODS =====================================================================

mod key;
mod mouse;
mod raw;
mod source;
mod state;

//= EXPORTS ==================================================================

pub use key::*;
pub use mouse::*;
pub(crate) use raw::*;
pub use source::*;
pub use state::*;

//= IMPORTS ==================================================================

use bitflags::bitflags;

//= INPUT FLAGS ==============================================================

bitflags! {
    #[derive(Clone, Debug, Default)]
    pub struct InputFlags: u8 {
        const ExtendedKey = 0b00000001;
        const Released = 0b00000010;
    }
}
