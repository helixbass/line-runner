use crate::Result;
use std::convert::TryFrom;
use wmidi::MidiMessage;

#[derive(Debug)]
pub struct Message {
    pub timestamp: u64,
    pub message: MidiMessage<'static>,
}

impl Message {
    pub fn new(timestamp: u64, message: MidiMessage<'static>) -> Self {
        Self { timestamp, message }
    }

    pub fn from(timestamp: u64, bytes: &[u8]) -> Result<Option<Self>> {
        Ok(MidiMessage::try_from(bytes)?
            .drop_unowned_sysex()
            .map(|message| Self::new(timestamp, message)))
    }
}
