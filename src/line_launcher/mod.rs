use midir::MidiOutputConnection;
use std::sync::mpsc::Receiver;
use wmidi::{Channel, MidiMessage, Note, Velocity};

use crate::BeatNumber;

enum State {
    NotPlaying,
    Playing { next_note_index: usize },
    WaitingToFireFinalNoteOff,
}

const CHANNEL: Channel = Channel::Ch1;
const VELOCITY: u8 = 100;

pub struct LineLauncher {
    beat_message_receiver: Receiver<BeatNumber>,
    state: State,
    notes: Vec<u8>,
    output: MidiOutputConnection,
}

impl LineLauncher {
    pub fn new(beat_message_receiver: Receiver<BeatNumber>, output: MidiOutputConnection) -> Self {
        Self {
            beat_message_receiver,
            state: State::NotPlaying,
            notes: vec![60, 53, 55, 58, 60, 61, 63, 65, 64],
            output,
        }
    }

    pub fn listen(&mut self) {
        loop {
            let beat_message = self.beat_message_receiver.recv().unwrap();
            match self.state {
                State::NotPlaying
                    if beat_message.quarter_note == 1 && beat_message.sixteenth_note == 1 =>
                {
                    self.state = State::Playing { next_note_index: 0 };
                    self.play_note();
                }
                State::Playing { next_note_index } => {
                    if next_note_index > 0 {
                        self.fire_note_off(next_note_index - 1);
                    }
                    self.play_note();
                }
                State::WaitingToFireFinalNoteOff if beat_message.sixteenth_note == 1 => {
                    self.fire_note_off(self.notes.len() - 1);
                    self.state = State::NotPlaying
                }
                _ => {}
            }
        }
    }

    fn play_note(&mut self) {
        if let State::Playing { next_note_index } = self.state {
            self.fire_note_on(next_note_index);

            self.state = if next_note_index == self.notes.len() - 1 {
                State::WaitingToFireFinalNoteOff
            } else {
                State::Playing {
                    next_note_index: next_note_index + 1,
                }
            };
        }
    }

    fn fire_note_on(&mut self, note_index: usize) {
        self.send_midi_message(MidiMessage::NoteOn(
            CHANNEL,
            Note::from_u8_lossy(self.get_note_number(note_index)),
            Velocity::from_u8_lossy(VELOCITY),
        ));
    }

    fn fire_note_off(&mut self, note_index: usize) {
        self.send_midi_message(MidiMessage::NoteOff(
            CHANNEL,
            Note::from_u8_lossy(self.get_note_number(note_index)),
            Velocity::from_u8_lossy(VELOCITY),
        ));
    }

    fn send_midi_message(&mut self, midi_message: MidiMessage) {
        let mut bytes_buffer = vec![0; midi_message.bytes_size()];
        midi_message.copy_to_slice(&mut bytes_buffer).unwrap();
        self.output.send(&bytes_buffer).unwrap();
    }

    fn get_note_number(&self, note_index: usize) -> u8 {
        self.notes[note_index]
    }
}
