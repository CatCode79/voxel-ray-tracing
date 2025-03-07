//= IMPORTS ==================================================================

use std::num::NonZeroU16;

//= EVENTS ===================================================================

pub enum Event {
    Resize {
        width: NonZeroU16,
        height: NonZeroU16,
    },
}
