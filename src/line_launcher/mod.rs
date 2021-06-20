use midir::MidiOutputConnection;
use rand::Rng;
use std::sync::mpsc::Receiver;
use wmidi::{Channel, MidiMessage, Note, Velocity};

use crate::{lines, BeatNumber, Line};

#[derive(Debug)]
enum State {
    NotPlaying,
    Playing {
        line_index: usize,
        next_note_index: usize,
    },
}

const CHANNEL: Channel = Channel::Ch1;
const VELOCITY: u8 = 100;

pub struct LineLauncher {
    beat_message_receiver: Receiver<BeatNumber>,
    state: State,
    lines: Vec<Line>,
    midi_message_sender: MidiMessageSender,
}

impl LineLauncher {
    pub fn new(beat_message_receiver: Receiver<BeatNumber>, output: MidiOutputConnection) -> Self {
        Self {
            beat_message_receiver,
            state: State::NotPlaying,
            lines: lines::all_lines(),
            midi_message_sender: MidiMessageSender { output },
        }
    }

    pub fn listen(&mut self) {
        loop {
            let beat_message = self.beat_message_receiver.recv().unwrap();
            match self.state {
                State::NotPlaying if beat_message.is_beginning_of_measure() => {
                    self.state = State::Playing {
                        line_index: rand::thread_rng().gen_range(0..self.lines.len()),
                        next_note_index: 0,
                    };
                    self.possibly_trigger_notes(beat_message);
                }
                State::Playing { .. } => {
                    self.possibly_trigger_notes(beat_message);
                }
                _ => {}
            }
        }
    }

    fn possibly_trigger_notes(&mut self, beat_message: BeatNumber) {
        match self.state {
            State::Playing {
                line_index,
                next_note_index,
            } => {
                let line = &self.lines[line_index];
                let mut did_trigger_note_off = false;
                if next_note_index > 0 {
                    let last_played_note = &line.notes[next_note_index - 1];
                    if beat_message.minus_sixteenths(last_played_note.duration)
                        == last_played_note.start
                    {
                        self.midi_message_sender
                            .fire_note_off(last_played_note.note);
                        did_trigger_note_off = true;
                    }
                }
                if next_note_index == line.notes.len() {
                    if did_trigger_note_off {
                        self.state = State::NotPlaying;
                    }
                    return;
                }
                let next_note = &line.notes[next_note_index];
                if beat_message == next_note.start {
                    self.midi_message_sender.fire_note_on(next_note.note);
                    self.state = State::Playing {
                        line_index,
                        next_note_index: next_note_index + 1,
                    }
                }
            }
            _ => {
                panic!(
                    "Called possibly_trigger_notes() while not playing: {:?}",
                    self.state
                );
            }
        }
    }
}

struct MidiMessageSender {
    output: MidiOutputConnection,
}

impl MidiMessageSender {
    fn fire_note_on(&mut self, note: Note) {
        self.send_midi_message(MidiMessage::NoteOn(
            CHANNEL,
            note,
            Velocity::from_u8_lossy(VELOCITY),
        ));
    }

    fn fire_note_off(&mut self, note: Note) {
        self.send_midi_message(MidiMessage::NoteOff(
            CHANNEL,
            note,
            Velocity::from_u8_lossy(VELOCITY),
        ));
    }

    fn send_midi_message(&mut self, midi_message: MidiMessage) {
        let mut bytes_buffer = vec![0; midi_message.bytes_size()];
        midi_message.copy_to_slice(&mut bytes_buffer).unwrap();
        self.output.send(&bytes_buffer).unwrap();
    }
}
