use midir::MidiOutputConnection;
use rand::Rng;
use std::sync::{
    mpsc::{Receiver, Sender},
    Arc, Mutex,
};
use std::time::{Duration, SystemTime};

use crate::{BeatNumber, Line, Progression};

mod midi_message_sender;
use midi_message_sender::MidiMessageSender;

mod note_off_triggerer;
use note_off_triggerer::{NoteOffInstruction, NoteOffTriggerer};

mod playing_state;
use playing_state::PlayingState;

mod progression_state;
use progression_state::ProgressionState;

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
