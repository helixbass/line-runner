pub mod midi;

use midi::Midi;

use serde::Deserialize;

use crate::Result;

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub midi: Midi,
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
        }
    }
}
