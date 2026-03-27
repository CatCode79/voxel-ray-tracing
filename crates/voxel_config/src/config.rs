//= IMPORTS ========================================================================================

use nanoserde::{DeJson, SerJson};

use std::fs::{read_to_string, write};

//= CONSTANTS ======================================================================================

// The shortest width reported by Steam HW Survey page to 2023 is 1024x768.
const MIN_WIDTH: u16 = 1920 / 2;

// The shortest height reported by Steam HW Survey page to 2023 is 1280x720.
const MIN_HEIGHT: u16 = 1080 / 2;

//= CONFIG =========================================================================================

/// Object with game configurations.
#[derive(Debug, DeJson, SerJson)]
pub struct Config {
    /// The minimum width of the drawable window.
    pub surface_width: u16,
    /// The minimum height of the drawable window.
    pub surface_height: u16,
    /// The window starts maximised and fullscreen-borderless otherwise
    /// windowed with decorations.
    pub maximized: bool,
}

impl Config {
    //- Load ---------------------------------------------------------------------------------------

    /// Loads a config file, path is chosen internally by some default paths.
    /// Returns a config object or error otherwise.
    pub(crate) fn load(filename: &str) -> Result<Self, String> {
        let filepath = filename;

        let contents = match read_to_string(filepath) {
            Ok(c) => c,
            Err(e) => return Err(format!("{e}: {filepath}")),
        };

        match DeJson::deserialize_json(contents.as_str()) {
            Ok(c) => Ok(c),
            Err(e) => Err(format!("{e}: {filepath}")),
        }
    }

    /// Loads a config file.
    ///
    /// If the file is not found then a Config object with default values
    /// returns.
    #[must_use]
    pub fn load_or_default(filename: &str) -> Self {
        Self::load(filename).unwrap_or_else(|e| {
            log::error!("{e:?}");
            Self::default()
        })
    }

    //- Save ---------------------------------------------------------------------------------------

    /// Saves a config file, path is chosen internally by some default paths.
    /// An error is returned if something went wrong.
    #[allow(dead_code)]
    pub(crate) fn save(&self, filename: &str) -> Result<(), String> {
        let filepath = filename;
        let contents = SerJson::serialize_json(self);
        match write(filepath, contents) {
            Ok(()) => Ok(()),
            Err(e) => Err(format!("{e}: {filepath}")),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            surface_width: MIN_WIDTH,
            surface_height: MIN_HEIGHT,
            maximized: false,
        }
    }
}
