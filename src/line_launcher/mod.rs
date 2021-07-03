use midir::MidiOutputConnection;
use rand::Rng;
use std::sync::mpsc::Receiver;
use wmidi::{Channel, MidiMessage, Note, Velocity};

use crate::{BeatNumber, Line, Progression};

#[derive(Clone, Copy, Debug)]
enum State {
    NotPlaying,
    Playing {
        line_index: usize,
        next_note_index: usize,
        pitch_offset: i8,
    },
}

const CHANNEL: Channel = Channel::Ch1;
const VELOCITY: u8 = 100;

pub struct LineLauncher {
    lines: Vec<Line>,
    pub progression: Progression,
}

impl LineLauncher {
    pub fn listen(
        &self,
        beat_message_receiver: Receiver<BeatNumber>,
        output: MidiOutputConnection,
    ) {
        let mut midi_message_sender = MidiMessageSender { output };
        let mut state = State::NotPlaying;
        let mut has_gotten_first_downbeat_message = false;
        let mut chord_index: usize = 0;
        loop {
            let beat_message = beat_message_receiver.recv().unwrap();
            if beat_message.is_beginning_of_measure() && has_gotten_first_downbeat_message {
                chord_index = (chord_index + 1) % self.progression.chords.len();
            }
            state = match state {
                State::NotPlaying if beat_message.is_beginning_of_measure() => {
                    state = State::Playing {
                        line_index: rand::thread_rng().gen_range(0..self.lines.len()),
                        next_note_index: 0,
                        pitch_offset: self.progression.chords[chord_index].pitch.index() as i8,
                    };
                    self.possibly_trigger_notes(state, &mut midi_message_sender, beat_message)
                }
                State::Playing { .. } => {
                    self.possibly_trigger_notes(state, &mut midi_message_sender, beat_message)
                }
                _ => state,
            };
            if !has_gotten_first_downbeat_message {
                has_gotten_first_downbeat_message = true;
            }
        }
    }

    fn possibly_trigger_notes(
        &self,
        state: State,
        midi_message_sender: &mut MidiMessageSender,
        beat_message: BeatNumber,
    ) -> State {
        match state {
            State::Playing {
                line_index,
                next_note_index,
                pitch_offset,
            } => {
                let line = &self.lines[line_index];
                let mut did_trigger_note_off = false;
                if next_note_index > 0 {
                    let last_played_note = &line.notes[next_note_index - 1];
                    if beat_message.minus_sixteenths(last_played_note.duration)
                        == last_played_note.start
                    {
                        midi_message_sender
                            .fire_note_off(last_played_note.note.step(pitch_offset).unwrap());
                        did_trigger_note_off = true;
                    }
                }
                if next_note_index == line.notes.len() {
                    return if did_trigger_note_off {
                        State::NotPlaying
                    } else {
                        state
                    };
                }
                let next_note = &line.notes[next_note_index];
                if beat_message == next_note.start {
                    midi_message_sender.fire_note_on(next_note.note.step(pitch_offset).unwrap());
                    return State::Playing {
                        line_index,
                        next_note_index: next_note_index + 1,
                        pitch_offset,
                    };
                }

                state
            }
            _ => {
                panic!(
                    "Called possibly_trigger_notes() while not playing: {:?}",
                    state
                );
            }
        }
    }
}

impl Default for LineLauncher {
    fn default() -> Self {
        Self {
            lines: Line::all(),
            progression: Progression::default(),
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
