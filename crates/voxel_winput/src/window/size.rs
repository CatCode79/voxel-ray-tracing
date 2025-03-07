//= WINDOW SIZE ==============================================================

#[derive(Clone, Copy, Debug)]
pub struct WindowSize {
    pub width: u16,
    pub height: u16,
}

impl WindowSize {
    pub fn new(width: u16, height: u16) -> Self {
        Self { width, height }
    }
}
