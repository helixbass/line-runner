use bus::Bus;
use log::*;
use midir::MidiOutputConnection;
use rand::Rng;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::{Duration, SystemTime};

use crate::{BeatNumber, Line, Message, MidiSlider, Progression};

mod midi_message_sender;
use midi_message_sender::MidiMessageSender;

mod note_off_scheduler;
use note_off_scheduler::{FireNoteOffMessage, NoteOffScheduler, ScheduleNoteOffMessage};

mod note_on_scheduler;
use note_on_scheduler::{FireNoteOnMessage, NoteOnScheduler, ScheduleNoteOnMessage};

mod playing_state;
use playing_state::PlayingState;

mod progression_state;
use progression_state::ProgressionState;

mod control_change_listener;
use control_change_listener::listen_for_control_changes;

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
    AheadOrBehindTheBeatRatioMessage(f64),
    FireNoteOnMessage(FireNoteOnMessage),
    FireNoteOffMessage(FireNoteOffMessage),
}

pub fn get_combined_message_receiver(
    beat_message_receiver: Receiver<BeatNumber>,
    duration_ratio_receiver: Receiver<f64>,
    ahead_or_behind_the_beat_ratio_receiver: Receiver<f64>,
    fire_note_on_receiver: Receiver<FireNoteOnMessage>,
    fire_note_off_receiver: Receiver<FireNoteOffMessage>,
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
    let fire_note_on_sender = sender.clone();
    thread::spawn(move || {
        for fire_note_on_message in fire_note_on_receiver.iter() {
            fire_note_on_sender
                .send(CombinedMessage::FireNoteOnMessage(fire_note_on_message))
                .unwrap();
        }
    });
    let fire_note_off_sender = sender.clone();
    thread::spawn(move || {
        for fire_note_off_message in fire_note_off_receiver.iter() {
            fire_note_off_sender
                .send(CombinedMessage::FireNoteOffMessage(fire_note_off_message))
                .unwrap();
        }
    });
    let duration_ratio_sender = sender.clone();
    thread::spawn(move || {
        for duration_ratio_message in duration_ratio_receiver.iter() {
            duration_ratio_sender
                .send(CombinedMessage::DurationRatioMessage(
                    duration_ratio_message,
                ))
                .unwrap();
        }
    });
    thread::spawn(move || {
        for ahead_or_behind_the_beat_ratio_message in ahead_or_behind_the_beat_ratio_receiver.iter()
        {
            sender
                .send(CombinedMessage::AheadOrBehindTheBeatRatioMessage(
                    ahead_or_behind_the_beat_ratio_message,
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
    pub fn from(progression: Progression) -> Self {
        Self {
            lines: Line::all(),
            progression,
        }
    }

    pub fn listen(
        &self,
        beat_message_receiver: Receiver<BeatNumber>,
        output: MidiOutputConnection,
        midi_messages: Option<Receiver<Message>>,
        duration_ratio_slider: Option<MidiSlider>,
        ahead_or_behind_the_beat_ratio_slider: Option<MidiSlider>,
    ) {
        let midi_message_sender = MidiMessageSender::new(output);
        let mut playing_state = PlayingState::NotPlaying;
        let mut progression_state = ProgressionState::new(&self.progression);
        let (note_on_scheduler, schedule_note_on_sender, fire_note_on_receiver) =
            NoteOnScheduler::new();
        note_on_scheduler.listen();
        let (note_off_scheduler, schedule_note_off_sender, fire_note_off_receiver) =
            NoteOffScheduler::new();
        note_off_scheduler.listen();
        let mut duration_between_sixteenth_notes = DurationBetweenSixteenthNotes::new();
        let mut midi_message_bus = Bus::new(100);
        let (mut duration_ratio, duration_ratio_receiver) = match duration_ratio_slider {
            Some(duration_ratio_slider) => (
                Some(1.0),
                listen_for_control_changes(midi_message_bus.add_rx(), duration_ratio_slider),
            ),
            None => (None, {
                let (_sender, receiver) = mpsc::channel();
                receiver
            }),
        };
        let (mut ahead_or_behind_the_beat_ratio, ahead_or_behind_the_beat_ratio_receiver) =
            match ahead_or_behind_the_beat_ratio_slider {
                Some(ahead_or_behind_the_beat_ratio_slider) => (
                    Some(0.5),
                    listen_for_control_changes(
                        midi_message_bus.add_rx(),
                        ahead_or_behind_the_beat_ratio_slider,
                    ),
                ),
                None => (None, {
                    let (_sender, receiver) = mpsc::channel();
                    receiver
                }),
            };
        if let Some(midi_messages) = midi_messages {
            thread::spawn(move || {
                for midi_message in midi_messages.iter() {
                    midi_message_bus.broadcast(midi_message);
                }
            });
        }
        for message in get_combined_message_receiver(
            beat_message_receiver,
            duration_ratio_receiver,
            ahead_or_behind_the_beat_ratio_receiver,
            fire_note_on_receiver,
            fire_note_off_receiver,
        )
        .iter()
        {
            match message {
                CombinedMessage::BeatMessage(beat_message) => {
                    debug!(
                        "beat message: {:?}, now: {:?}",
                        beat_message,
                        SystemTime::now()
                    );
                    duration_between_sixteenth_notes =
                        duration_between_sixteenth_notes.process_beat_message(&beat_message);
                    if (!progression_state.has_started() && beat_message.is_beginning_of_measure())
                        || beat_message.is_next_beginning_of_measure()
                    {
                        progression_state.tick_measure();
                    }
                    match playing_state {
                        PlayingState::NotPlaying
                            if beat_message.is_next_beginning_of_measure()
                                && matches!(
                                    duration_between_sixteenth_notes.get_duration(),
                                    Some(_)
                                ) =>
                        {
                            playing_state = PlayingState::Playing {
                                line_index: rand::thread_rng().gen_range(0..self.lines.len()),
                                next_note_index: 0,
                                pitch_offset: progression_state.current_chord().pitch.index(),
                                next_note_off_index: 0,
                            };
                            self.possibly_schedule_note_on(
                                &mut playing_state,
                                beat_message,
                                &schedule_note_on_sender,
                                &duration_between_sixteenth_notes,
                                &ahead_or_behind_the_beat_ratio,
                            );
                        }
                        PlayingState::Playing { .. } => {
                            self.possibly_schedule_note_on(
                                &mut playing_state,
                                beat_message,
                                &schedule_note_on_sender,
                                &duration_between_sixteenth_notes,
                                &ahead_or_behind_the_beat_ratio,
                            );
                        }
                        _ => (),
                    };
                }
                CombinedMessage::DurationRatioMessage(new_duration_ratio) => {
                    debug!("duration ratio change: {}", new_duration_ratio);
                    duration_ratio = Some(new_duration_ratio);
                }
                CombinedMessage::AheadOrBehindTheBeatRatioMessage(
                    new_ahead_or_behind_the_beat_ratio,
                ) => {
                    debug!(
                        "ahead or behind the beat ratio change: {}",
                        new_ahead_or_behind_the_beat_ratio
                    );
                    ahead_or_behind_the_beat_ratio = Some(new_ahead_or_behind_the_beat_ratio);
                }
                CombinedMessage::FireNoteOffMessage(fire_note_off_message) => match playing_state {
                    PlayingState::Playing {
                        next_note_off_index,
                        line_index,
                        next_note_index,
                        pitch_offset,
                    } if next_note_off_index <= fire_note_off_message.note_index => {
                        let line = &self.lines[line_index];
                        let note = &line.notes[fire_note_off_message.note_index];
                        let note_with_offset = note.note.step(pitch_offset).unwrap();

                        debug!("Firing note off: {:?}", fire_note_off_message);
                        midi_message_sender.fire_note_off(note_with_offset);

                        playing_state = if fire_note_off_message.note_index == line.notes.len() - 1
                        {
                            PlayingState::NotPlaying
                        } else {
                            PlayingState::Playing {
                                line_index,
                                next_note_index,
                                pitch_offset,
                                next_note_off_index: fire_note_off_message.note_index,
                            }
                        };
                    }
                    _ => {
                        if let PlayingState::Playing {
                            next_note_off_index,
                            ..
                        } = playing_state
                        {
                            debug!(
                                "Received fire note off message with note index < next_note_off_index ({}): {:?}",
                                next_note_off_index,
                                fire_note_off_message
                            );
                        } else {
                            debug!(
                                "Received fire note off message while not playing: {:?}",
                                fire_note_off_message
                            );
                        }
                    }
                },
                CombinedMessage::FireNoteOnMessage(fire_note_on_message) => match playing_state {
                    PlayingState::Playing {
                        next_note_off_index,
                        line_index,
                        next_note_index,
                        pitch_offset,
                    } => {
                        let line = &self.lines[line_index];
                        let mut updated_next_note_off_index: Option<usize> = None;
                        debug!("Received fire note on message: {:?}", fire_note_on_message);
                        if fire_note_on_message.note_index > 0
                            && next_note_off_index < fire_note_on_message.note_index
                        {
                            debug!(
                                "Synchronously firing note off's, next_note_off_index: {}",
                                next_note_off_index
                            );
                            for note_index in next_note_off_index..fire_note_on_message.note_index {
                                let note = &line.notes[note_index];
                                midi_message_sender
                                    .fire_note_off(note.note.step(pitch_offset).unwrap());
                                updated_next_note_off_index = Some(note_index + 1);
                            }
                        }
                        let use_next_note_off_index =
                            updated_next_note_off_index.unwrap_or(next_note_off_index);
                        let note = &line.notes[fire_note_on_message.note_index];
                        let note_with_offset = note.note.step(pitch_offset).unwrap();
                        midi_message_sender.fire_note_on(note_with_offset);
                        if let Some(duration_ratio) = duration_ratio {
                            if let Some(duration_between_sixteenth_notes) =
                                duration_between_sixteenth_notes.get_duration()
                            {
                                debug!(
                                    "Sending schedule note off message, note_index: {}, now: {:?}, duration_ratio: {}",
                                    fire_note_on_message.note_index, SystemTime::now(), duration_ratio
                                );
                                schedule_note_off_sender
                                    .send(ScheduleNoteOffMessage {
                                        time: SystemTime::now()
                                            + duration_between_sixteenth_notes
                                                .mul_f64(duration_ratio),
                                        note_index: fire_note_on_message.note_index,
                                    })
                                    .unwrap();
                            }
                        }
                        playing_state = if use_next_note_off_index >= line.notes.len() {
                            PlayingState::NotPlaying
                        } else {
                            PlayingState::Playing {
                                line_index,
                                next_note_index,
                                pitch_offset,
                                next_note_off_index: use_next_note_off_index,
                            }
                        };
                    }
                    _ => {
                        panic!(
                            "Received fire note on message while not playing: {:?}",
                            fire_note_on_message
                        );
                    }
                },
            }
        }
    }

    fn possibly_schedule_note_on(
        &self,
        playing_state: &mut PlayingState,
        beat_message: BeatNumber,
        schedule_note_on_sender: &Sender<ScheduleNoteOnMessage>,
        duration_between_sixteenth_notes: &DurationBetweenSixteenthNotes,
        ahead_or_behind_the_beat_ratio: &Option<f64>,
    ) {
        match *playing_state {
            PlayingState::Playing {
                line_index,
                next_note_index,
                pitch_offset,
                next_note_off_index,
            } => {
                let line = &self.lines[line_index];
                if next_note_index >= line.notes.len() {
                    return;
                }
                let next_note = &line.notes[next_note_index];
                if beat_message.add_sixteenths(1) == next_note.start {
                    if let Some(duration_between_sixteenth_notes) =
                        duration_between_sixteenth_notes.get_duration()
                    {
                        debug!(
                            "Sending schedule note on message, note_index: {}, now: {:?}",
                            next_note_index,
                            SystemTime::now()
                        );
                        schedule_note_on_sender
                            .send(ScheduleNoteOnMessage {
                                time: SystemTime::now()
                                    + duration_between_sixteenth_notes.mul_f64(
                                        1.0 + ((0.5
                                            - ahead_or_behind_the_beat_ratio.unwrap_or(0.5))
                                            * 2.0),
                                    ),
                                note_index: next_note_index,
                            })
                            .unwrap();

                        *playing_state = PlayingState::Playing {
                            line_index,
                            next_note_index: next_note_index + 1,
                            pitch_offset,
                            next_note_off_index,
                        };
                    } else {
                        panic!(
                            "Couldn't schedule note on because no duration between sixteenth notes"
                        );
                    }
                }
            }
            _ => {
                panic!(
                    "Called possibly_schedule_note_on() while not playing: {:?}",
                    playing_state
                );
            }
        }
    }
}
