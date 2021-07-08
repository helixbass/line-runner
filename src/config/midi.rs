use crate::Result;
// use serde::{Deserialize, Deserializer};
use std::convert::TryInto;
use wmidi::{Channel, ControlFunction, U7};

pub struct Midi {
    pub port: Option<String>,
}

impl Default for Midi {
    fn default() -> Self {
        Self { port: None }
    }
}

#[derive(Copy, Clone, Debug /*Deserialize*/)]
pub struct MidiSlider {
    // #[serde(deserialize_with = "deserialize_channel")]
    pub channel: Channel,
    // #[serde(deserialize_with = "deserialize_control_function")]
    pub control_change: ControlFunction,
}

// fn deserialize_channel<'de, TDeserializer>(
//     deserializer: TDeserializer,
// ) -> std::result::Result<Channel, TDeserializer::Error>
// where
//     TDeserializer: Deserializer<'de>,
// {
//     let channel_number: u8 = Deserialize::deserialize(deserializer)?;
//     Channel::from_index(channel_number - 1).map_err(serde::de::Error::custom)
// }

// fn deserialize_control_function<'de, TDeserializer>(
//     deserializer: TDeserializer,
// ) -> std::result::Result<ControlFunction, TDeserializer::Error>
// where
//     TDeserializer: Deserializer<'de>,
// {
//     let value_u8: u8 = Deserialize::deserialize(deserializer)?;
//     u8_to_control_function(value_u8).map_err(serde::de::Error::custom)?
// }

pub fn u8_to_control_function(value: u8) -> Result<ControlFunction> {
    let value_u7: U7 = value.try_into()?;
    Ok(value_u7.into())
}
