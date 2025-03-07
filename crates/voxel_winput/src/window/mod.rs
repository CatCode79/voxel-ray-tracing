//= MODULES ==================================================================

mod event;
mod monitor;
mod size;
mod window;

//= EXPORTS ==================================================================

pub use event::*;
pub use monitor::*;
pub use size::*;
pub use window::*;

//= CONSTANTS ================================================================

pub const DEFAULT_FRAMERATE: f64 = 30.0;
