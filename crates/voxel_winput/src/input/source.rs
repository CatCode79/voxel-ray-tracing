//= IMPORTS ==================================================================

use crate::input::{KeyCode, MouseButton};

//= ENUM INPUT SOURCE ========================================================

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum InputSource {
    Key { source: KeyCode },
    Mouse { source: MouseButton },
    //Joy { source: JoypadButton },
}
