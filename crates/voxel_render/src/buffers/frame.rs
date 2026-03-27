//= FRAME BUFFER ===================================================================================

#[derive(Debug, Default)]
pub struct FrameData {
    counter: u32,     // Used to randomise on shaders
    accumulator: u32, // Used to reset the info accumulation on shaders
}

impl FrameData {
    pub(crate) const fn increment(&mut self) {
        self.counter += 1;
        self.accumulator += 1;
    }

    pub(crate) const fn reset(&mut self) {
        self.accumulator = 0;
    }
}
