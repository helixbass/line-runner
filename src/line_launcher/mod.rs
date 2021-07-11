use bus::Bus;
use midir::MidiOutputConnection;
use rand::Rng;
use std::sync::{
    mpsc::{self, Receiver, Sender},
    Arc, Mutex,
};
use std::thread;
use std::time::{Duration, SystemTime};

use crate::{BeatNumber, Line, Message, Progression};

mod midi_message_sender;
use midi_message_sender::MidiMessageSender;

mod note_off_triggerer;
use note_off_triggerer::{NoteOffInstruction, NoteOffTriggerer};

mod playing_state;
use playing_state::PlayingState;

mod progression_state;
use progression_state::ProgressionState;

mod duration_slider_listener;
use duration_slider_listener::listen_for_duration_control_changes;

enum DurationBetweenSixteenthNotes {
    Uninitialized,
    PartiallyInitialized {
        last_timestamp: SystemTime,
    },
    Initialized {
        last_timestamp: SystemTime,
        last_duration: Duration,
    },
}

impl DurationBetweenSixteenthNotes {
    pub fn new() -> Self {
        Self::Uninitialized
    }

    pub fn process_beat_message(&self, _beat_message: &BeatNumber) -> Self {
        let now = SystemTime::now();
        match self {
            Self::Uninitialized => Self::PartiallyInitialized {
                last_timestamp: now,
            },
            Self::PartiallyInitialized { last_timestamp }
            | Self::Initialized { last_timestamp, .. } => Self::Initialized {
                last_timestamp: now,
                last_duration: now.duration_since(*last_timestamp).unwrap(),
            },
        }
    }

    pub fn get_duration(&self) -> Option<Duration> {
        match self {
            Self::Initialized { last_duration, .. } => Some(*last_duration),
            _ => None,
        }
    }
}

pub enum CombinedMessage {
    BeatMessage(BeatNumber),
    DurationRatioMessage(f64),
}

pub fn get_combined_message_receiver(
    beat_message_receiver: Receiver<BeatNumber>,
    duration_ratio_receiver: Receiver<f64>,
) -> Receiver<CombinedMessage> {
    let (sender, receiver) = mpsc::channel();
    let beat_message_sender = sender.clone();
    thread::spawn(move || {
        for beat_message in beat_message_receiver.iter() {
            beat_message_sender
                .send(CombinedMessage::BeatMessage(beat_message))
                .unwrap();
        }
    });
    thread::spawn(move || {
        for duration_ratio_message in duration_ratio_receiver.iter() {
            sender
                .send(CombinedMessage::DurationRatioMessage(
                    duration_ratio_message,
                ))
                .unwrap();
        }
    });
    receiver
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
        midi_messages: Receiver<Message>,
    ) {
        let midi_message_sender = MidiMessageSender::new(output);
        let state_mutex = Arc::new(Mutex::new(PlayingState::NotPlaying));
        let mut progression_state = ProgressionState::new(&self.progression);
        let (note_off_triggerer, note_off_sender) =
            NoteOffTriggerer::new(midi_message_sender.clone(), state_mutex.clone());
        note_off_triggerer.listen();
        let mut duration_between_sixteenth_notes = DurationBetweenSixteenthNotes::new();
        let mut duration_ratio = 1.0;
        let mut midi_message_bus = Bus::new(100);
        let duration_ratio_receiver =
            listen_for_duration_control_changes(midi_message_bus.add_rx());
        thread::spawn(move || {
            for midi_message in midi_messages.iter() {
                midi_message_bus.broadcast(midi_message);
            }
        });
        for message in
            get_combined_message_receiver(beat_message_receiver, duration_ratio_receiver).iter()
        {
            match message {
                CombinedMessage::BeatMessage(beat_message) => {
                    duration_between_sixteenth_notes =
                        duration_between_sixteenth_notes.process_beat_message(&beat_message);
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
                                &duration_between_sixteenth_notes,
                                duration_ratio,
                            )
                        }
                        PlayingState::Playing { .. } => self.possibly_trigger_notes(
                            *state,
                            &midi_message_sender,
                            beat_message,
                            &note_off_sender,
                            &duration_between_sixteenth_notes,
                            duration_ratio,
                        ),
                        _ => *state,
                    };
                }
                CombinedMessage::DurationRatioMessage(new_duration_ratio) => {
                    duration_ratio = new_duration_ratio;
                }
            }
        }
    }

    fn possibly_trigger_notes(
        &self,
        state: PlayingState,
        midi_message_sender: &MidiMessageSender,
        beat_message: BeatNumber,
        note_off_sender: &Sender<NoteOffInstruction>,
        duration_between_sixteenth_notes: &DurationBetweenSixteenthNotes,
        duration_ratio: f64,
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
                    if let Some(duration_between_sixteenth_notes) =
                        duration_between_sixteenth_notes.get_duration()
                    {
                        note_off_sender
                            .send(NoteOffInstruction {
                                note: next_note_with_offset,
                                time: SystemTime::now()
                                    + duration_between_sixteenth_notes.mul_f64(duration_ratio),
                                note_index: next_note_index,
                            })
                            .unwrap();
                    }
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
