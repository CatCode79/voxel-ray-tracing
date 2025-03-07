//= FRAME BUFFER =============================================================

#[derive(Debug, Default)]
pub(crate) struct FrameData {
    counter: u32,   // Used to randomize on shaders
    accumulator: u32, // Used to reset the info accumulation on shaders
}

impl FrameData {
    pub(crate) fn increment(&mut self) {
        self.counter += 1;
        self.accumulator += 1;
    }

    pub(crate) fn reset(&mut self) {
        self.accumulator = 0;
    }
}
