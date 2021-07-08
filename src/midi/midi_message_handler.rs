use crate::{Message, Result};

pub trait MidiMessageHandler: Send + Sync {
    fn handle_midi_messages(&self, midi_messages: &[Message]) -> Result<()>;
}
