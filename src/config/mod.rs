pub mod midi;

use midi::Midi;

use serde::Deserialize;

use crate::{Progression, Result};

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub midi: Midi,
    #[serde(default)]
    pub progression: Progression,
}

impl Config {
    pub fn from(yaml: &str) -> Result<Config> {
        Ok(serde_yaml::from_str(yaml)?)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            midi: Midi::default(),
            progression: Progression::default(),
        }
    }
}
