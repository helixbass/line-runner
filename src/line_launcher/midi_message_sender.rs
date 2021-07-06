use midir::MidiOutputConnection;
use std::sync::{Arc, Mutex};
use wmidi::{Channel, MidiMessage, Note, Velocity};

const CHANNEL: Channel = Channel::Ch1;
const VELOCITY: u8 = 100;

#[derive(Clone)]
pub struct MidiMessageSender {
    output: Arc<Mutex<MidiOutputConnection>>,
}

impl MidiMessageSender {
    pub fn new(output: MidiOutputConnection) -> Self {
        Self {
            output: Arc::new(Mutex::new(output)),
        }
    }

    pub fn fire_note_on(&self, note: Note) {
        self.send_midi_message(MidiMessage::NoteOn(
            CHANNEL,
            note,
            Velocity::from_u8_lossy(VELOCITY),
        ));
    }

    pub fn fire_note_off(&self, note: Note) {
        self.send_midi_message(MidiMessage::NoteOff(
            CHANNEL,
            note,
            Velocity::from_u8_lossy(VELOCITY),
        ));
    }

    pub fn send_midi_message(&self, midi_message: MidiMessage) {
        let mut bytes_buffer = vec![0; midi_message.bytes_size()];
        midi_message.copy_to_slice(&mut bytes_buffer).unwrap();
        self.output.lock().unwrap().send(&bytes_buffer).unwrap();
    }
}
