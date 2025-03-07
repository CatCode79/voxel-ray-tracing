//= IMPORTS ==================================================================

use crate::input::InputFlags;

//= INPUT STATE ==============================================================

#[derive(Clone, Debug, Default)]
pub struct InputState {
    //pub virtual_key: u16,
    x_coord: Option<i16>,
    y_coord: Option<i16>,
    flags: InputFlags,
    pressure_time: u8, // First pressure time has value 0, maximum is 255, after that is always 255  // TODO: BUG! is frame dependant!
                       //pub pressure_force: f32,
}

impl InputState {
    pub fn new(x: Option<i16>, y: Option<i16>) -> Self {
        Self {
            x_coord: x,
            y_coord: y,
            flags: InputFlags::empty(),
            pressure_time: 0,
        }
    }

    //- Position Related Methods ---------------------------------------------

    pub fn set_coords(&mut self, x: i16, y: i16) {
        self.x_coord = Some(x);
        self.y_coord = Some(y);
    }

    //- Flags Related Methods ------------------------------------------------

    #[inline(always)]
    pub fn has_flag(&self, flag: InputFlags) -> bool {
        self.flags.contains(flag)
    }

    #[inline(always)]
    pub fn set_flag(&mut self, flag: InputFlags) {
        self.flags.set(flag, true);
    }

    #[inline(always)]
    pub fn clear_flag(&mut self, flag: InputFlags) {
        self.flags.set(flag, false);
    }

    #[inline(always)]
    pub fn toggle_flag(&mut self, flag: InputFlags) {
        self.flags.toggle(flag);
    }

    //- Pressure Time Related methods ----------------------------------------

    #[inline(always)]
    pub fn pressure_time(&self) -> u8 {
        self.pressure_time
    }

    #[inline(always)]
    pub fn increment_pressure_time(&mut self) {
        self.pressure_time = self.pressure_time.saturating_add(1);
    }
}
