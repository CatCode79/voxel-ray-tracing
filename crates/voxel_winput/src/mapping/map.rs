//= IMPORTS ==================================================================

use crate::input::InputSource;
use crate::mapping::kind::InputKind;

//= INPUT MAP ================================================================

#[derive(Debug)]
pub struct InputMapping {
    primary: Vec<Option<InputSource>>,
    secondary: Vec<Option<InputSource>>,
}

impl InputMapping {
    pub fn new() -> Self {
        Self {
            primary: vec![None; InputKind::Max as usize],
            secondary: vec![None; InputKind::Max as usize],
        }
    }

    //- Getters --------------------------------------------------------------

    pub fn get_primary(&self, kind: InputKind) -> Option<&InputSource> {
        self.primary[kind as usize].as_ref()
    }

    pub fn get_secondary(&self, kind: InputKind) -> Option<&InputSource> {
        self.secondary[kind as usize].as_ref()
    }

    //- Setters --------------------------------------------------------------

    pub fn set_primary(&mut self, kind: InputKind, source: InputSource) {
        self.primary[kind as usize] = Some(source);
    }

    pub fn set_secondary(&mut self, kind: InputKind, source: InputSource) {
        self.secondary[kind as usize] = Some(source);
    }
}
