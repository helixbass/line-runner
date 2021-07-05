use midir::MidiOutputConnection;
use rand::Rng;
use std::sync::{mpsc::Receiver, Arc, Mutex};
use wmidi::{Channel, MidiMessage, Note, Velocity};

use crate::{BeatNumber, Chord, Line, Progression};

#[derive(Clone, Copy, Debug)]
enum PlayingState {
    NotPlaying,
    Playing {
        line_index: usize,
        next_note_index: usize,
        pitch_offset: i8,
        has_fired_previous_note_off: bool,
    },
}

enum ProgressionChordIndexState {
    HaventStarted,
    AtChordIndex(usize),
}

struct ProgressionState<'progression> {
    progression: &'progression Progression,
    chord_index_state: ProgressionChordIndexState,
}

impl<'progression> ProgressionState<'progression> {
    pub fn new(progression: &'progression Progression) -> Self {
        Self {
            progression,
            chord_index_state: ProgressionChordIndexState::HaventStarted,
        }
    }

    pub fn chord_index(&self) -> usize {
        if let ProgressionChordIndexState::AtChordIndex(chord_index) = self.chord_index_state {
            chord_index
        } else {
            0
        }
    }

    pub fn current_chord(&self) -> &Chord {
        &self.progression.chords[self.chord_index()]
    }

    pub fn tick_measure(&mut self) {
        self.chord_index_state = match self.chord_index_state {
            ProgressionChordIndexState::HaventStarted => {
                ProgressionChordIndexState::AtChordIndex(0)
            }
            ProgressionChordIndexState::AtChordIndex(chord_index) => {
                ProgressionChordIndexState::AtChordIndex(
                    (chord_index + 1) % self.progression.chords.len(),
                )
            }
        }
    }
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
        let midi_message_sender = Arc::new(Mutex::new(MidiMessageSender { output }));
        let state_mutex = Arc::new(Mutex::new(PlayingState::NotPlaying));
        let mut progression_state = ProgressionState::new(&self.progression);
        loop {
            let beat_message = beat_message_receiver.recv().unwrap();
            if beat_message.is_beginning_of_measure() {
                progression_state.tick_measure();
            }
            let mut state = state_mutex.lock().unwrap();
            *state = match *state {
                PlayingState::NotPlaying if beat_message.is_beginning_of_measure() => {
                    *state = PlayingState::Playing {
                        line_index: rand::thread_rng().gen_range(0..self.lines.len()),
                        next_note_index: 0,
                        pitch_offset: progression_state.current_chord().pitch.index(),
                        has_fired_previous_note_off: true,
                    };
                    self.possibly_trigger_notes(*state, &midi_message_sender, beat_message)
                }
                PlayingState::Playing { .. } => {
                    self.possibly_trigger_notes(*state, &midi_message_sender, beat_message)
                }
                _ => *state,
            };
        }
    }

    fn possibly_trigger_notes(
        &self,
        state: PlayingState,
        midi_message_sender: &Arc<Mutex<MidiMessageSender>>,
        beat_message: BeatNumber,
    ) -> PlayingState {
        match state {
            PlayingState::Playing {
                line_index,
                next_note_index,
                pitch_offset,
                has_fired_previous_note_off,
            } => {
                let line = &self.lines[line_index];
                let mut did_trigger_note_off = false;
                if next_note_index > 0 {
                    let last_played_note = &line.notes[next_note_index - 1];
                    if !has_fired_previous_note_off
                        && beat_message.minus_sixteenths(last_played_note.duration)
                            == last_played_note.start
                    {
                        midi_message_sender
                            .lock()
                            .unwrap()
                            .fire_note_off(last_played_note.note.step(pitch_offset).unwrap());
                        did_trigger_note_off = true;
                    }
                }
                if next_note_index == line.notes.len() {
                    return if did_trigger_note_off {
                        PlayingState::NotPlaying
                    } else {
                        state
                    };
                }
                let next_note = &line.notes[next_note_index];
                if beat_message == next_note.start {
                    midi_message_sender
                        .lock()
                        .unwrap()
                        .fire_note_on(next_note.note.step(pitch_offset).unwrap());
                    return PlayingState::Playing {
                        line_index,
                        next_note_index: next_note_index + 1,
                        pitch_offset,
                        has_fired_previous_note_off: false,
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

impl From<Progression> for LineLauncher {
    fn from(progression: Progression) -> Self {
        Self {
            lines: Line::all(),
            progression,
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
