use serde::{Deserialize, Deserializer};
use std::convert::TryInto;
use wmidi::{Channel, ControlFunction, U7};

#[derive(Debug, Deserialize)]
pub struct Midi {
    pub port: Option<String>,
    pub duration_ratio_slider: Option<MidiSlider>,
}

impl Default for Midi {
    fn default() -> Self {
        Self {
            port: None,
            duration_ratio_slider: None,
        }
    }
}

#[derive(Copy, Clone, Debug, Deserialize)]
pub struct MidiSlider {
    #[serde(deserialize_with = "deserialize_channel")]
    pub channel: Channel,
    #[serde(deserialize_with = "deserialize_control_function")]
    pub control_change: ControlFunction,
}

fn deserialize_channel<'de, TDeserializer>(
    deserializer: TDeserializer,
) -> std::result::Result<Channel, TDeserializer::Error>
where
    TDeserializer: Deserializer<'de>,
{
    let channel_number: u8 = Deserialize::deserialize(deserializer)?;
    Channel::from_index(channel_number - 1).map_err(serde::de::Error::custom)
}

fn deserialize_control_function<'de, TDeserializer>(
    deserializer: TDeserializer,
) -> std::result::Result<ControlFunction, TDeserializer::Error>
where
    TDeserializer: Deserializer<'de>,
{
    let value_u8: u8 = Deserialize::deserialize(deserializer)?;
    let value_u7: U7 = value_u8.try_into().map_err(serde::de::Error::custom)?;
    Ok(value_u7.into())
}
