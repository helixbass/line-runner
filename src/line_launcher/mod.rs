use midir::MidiOutputConnection;
use rand::Rng;
use std::sync::{
    mpsc::{self, Receiver, Sender},
    Arc, Mutex,
};
use std::thread::{self, JoinHandle};
use std::time::{Duration, SystemTime};
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

struct NoteOffInstruction {
    note: Note,
    time: SystemTime,
    note_index: usize,
}

struct NoteOffTriggerer {
    receiver: Receiver<NoteOffInstruction>,
    midi_message_sender: MidiMessageSender,
    playing_state: Arc<Mutex<PlayingState>>,
}

impl NoteOffTriggerer {
    pub fn new(
        midi_message_sender: MidiMessageSender,
        playing_state: Arc<Mutex<PlayingState>>,
    ) -> (Self, Sender<NoteOffInstruction>) {
        let (sender, receiver) = mpsc::channel();

        let note_off_triggerer = Self {
            receiver,
            midi_message_sender,
            playing_state,
        };

        (note_off_triggerer, sender)
    }

    pub fn listen(self) -> JoinHandle<()> {
        thread::spawn(move || loop {
            let note_off_instruction = self.receiver.recv().unwrap();

            let now = SystemTime::now();
            let from_now = note_off_instruction
                .time
                .duration_since(now)
                .unwrap_or_default();

            spin_sleep::sleep(from_now);

            let mut playing_state = self.playing_state.lock().unwrap();

            match *playing_state {
                PlayingState::Playing {
                    has_fired_previous_note_off: false,
                    line_index,
                    next_note_index,
                    pitch_offset,
                } if next_note_index == note_off_instruction.note_index + 1 => {
                    self.midi_message_sender
                        .fire_note_off(note_off_instruction.note);

                    *playing_state = PlayingState::Playing {
                        line_index,
                        next_note_index,
                        pitch_offset,
                        has_fired_previous_note_off: true,
                    };
                }
                _ => (),
            }
        })
    }
}

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
        let midi_message_sender = MidiMessageSender::new(output);
        let state_mutex = Arc::new(Mutex::new(PlayingState::NotPlaying));
        let mut progression_state = ProgressionState::new(&self.progression);
        let (note_off_triggerer, note_off_sender) =
            NoteOffTriggerer::new(midi_message_sender.clone(), state_mutex.clone());
        note_off_triggerer.listen();
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
                    self.possibly_trigger_notes(
                        *state,
                        &midi_message_sender,
                        beat_message,
                        &note_off_sender,
                    )
                }
                PlayingState::Playing { .. } => self.possibly_trigger_notes(
                    *state,
                    &midi_message_sender,
                    beat_message,
                    &note_off_sender,
                ),
                _ => *state,
            };
        }
    }

    fn possibly_trigger_notes(
        &self,
        state: PlayingState,
        midi_message_sender: &MidiMessageSender,
        beat_message: BeatNumber,
        note_off_sender: &Sender<NoteOffInstruction>,
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
                            .fire_note_off(last_played_note.note.step(pitch_offset).unwrap());
                        did_trigger_note_off = true;
                    }
                }
                if next_note_index == line.notes.len() {
                    return if did_trigger_note_off || has_fired_previous_note_off {
                        PlayingState::NotPlaying
                    } else {
                        state
                    };
                }
                let next_note = &line.notes[next_note_index];
                if beat_message == next_note.start {
                    let next_note_with_offset = next_note.note.step(pitch_offset).unwrap();
                    midi_message_sender.fire_note_on(next_note_with_offset);
                    note_off_sender
                        .send(NoteOffInstruction {
                            note: next_note_with_offset,
                            time: SystemTime::now() + Duration::from_millis(50),
                            note_index: next_note_index,
                        })
                        .unwrap();
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

#[derive(Clone)]
struct MidiMessageSender {
    output: Arc<Mutex<MidiOutputConnection>>,
}

impl MidiMessageSender {
    fn new(output: MidiOutputConnection) -> Self {
        Self {
            output: Arc::new(Mutex::new(output)),
        }
    }

    fn fire_note_on(&self, note: Note) {
        self.send_midi_message(MidiMessage::NoteOn(
            CHANNEL,
            note,
            Velocity::from_u8_lossy(VELOCITY),
        ));
    }

    fn fire_note_off(&self, note: Note) {
        self.send_midi_message(MidiMessage::NoteOff(
            CHANNEL,
            note,
            Velocity::from_u8_lossy(VELOCITY),
        ));
    }

    fn send_midi_message(&self, midi_message: MidiMessage) {
        let mut bytes_buffer = vec![0; midi_message.bytes_size()];
        midi_message.copy_to_slice(&mut bytes_buffer).unwrap();
        self.output.lock().unwrap().send(&bytes_buffer).unwrap();
    }
}
