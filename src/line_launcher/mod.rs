use bus::Bus;
use log::*;
use midir::MidiOutputConnection;
use rand::{prelude::ThreadRng, Rng};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::{Duration, SystemTime};

use crate::{BeatNumber, Line, LineNote, Message, MidiSlider, Progression};

mod midi_message_sender;
use midi_message_sender::MidiMessageSender;

mod note_off_scheduler;
use note_off_scheduler::{FireNoteOffMessage, NoteOffScheduler, ScheduleNoteOffMessage};

mod note_on_scheduler;
use note_on_scheduler::{FireNoteOnMessage, NoteOnScheduler, ScheduleNoteOnMessage};

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
    RandomizeNoteStartTimeRatioMessage(f64),
    FireNoteOnMessage(FireNoteOnMessage),
    FireNoteOffMessage(FireNoteOffMessage),
}

pub fn get_combined_message_receiver(
    beat_message_receiver: Receiver<BeatNumber>,
    duration_ratio_receiver: Receiver<f64>,
    ahead_or_behind_the_beat_ratio_receiver: Receiver<f64>,
    randomize_note_start_time_ratio_receiver: Receiver<f64>,
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
    let randomize_note_start_time_ratio_sender = sender.clone();
    thread::spawn(move || {
        for randomize_note_start_time_ratio_message in
            randomize_note_start_time_ratio_receiver.iter()
        {
            randomize_note_start_time_ratio_sender
                .send(CombinedMessage::RandomizeNoteStartTimeRatioMessage(
                    randomize_note_start_time_ratio_message,
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

#[derive(Clone, PartialEq, Eq, Debug, Copy)]
struct MeasureBeat {
    beat_number: BeatNumber,
    measure: u32,
}

impl MeasureBeat {
    pub fn new(beat_number: BeatNumber, measure: u32) -> Self {
        Self {
            beat_number,
            measure,
        }
    }

    pub fn increment(&mut self) {
        if self.beat_number.sixteenth_note == 15 {
            self.measure += 1;
        }
        self.beat_number.increment();
    }

    pub fn incremented(&self) -> Self {
        let mut cloned = *self;
        cloned.increment();
        cloned
    }

    pub fn incremented_by(&self, num_sixteenths: u32) -> Self {
        let mut cloned = *self;
        for _ in 0..num_sixteenths {
            cloned.increment();
        }
        cloned
    }
}

impl Default for MeasureBeat {
    fn default() -> Self {
        Self {
            beat_number: BeatNumber::new(0),
            measure: 0,
        }
    }
}

#[derive(Debug)]
struct NoteAndMeasureBeat {
    note: wmidi::Note,
    measure_beat: MeasureBeat,
}

#[derive(Debug)]
struct PlannedNote {
    note_on: NoteAndMeasureBeat,
    note_off: NoteAndMeasureBeat,
    has_note_on_fired: bool,
    has_note_off_fired: bool,
}

impl PlannedNote {
    pub fn new(line_note: &LineNote, measure_beat_start: MeasureBeat, pitch_offset: i8) -> Self {
        let note_adjusted = line_note.note.step(pitch_offset).unwrap();
        Self {
            note_on: NoteAndMeasureBeat {
                note: note_adjusted,
                measure_beat: measure_beat_start,
            },
            note_off: NoteAndMeasureBeat {
                note: note_adjusted,
                measure_beat: measure_beat_start.incremented_by(line_note.duration),
            },
            has_note_on_fired: false,
            has_note_off_fired: false,
        }
    }
}

#[derive(Debug)]
struct PlannedNotes {
    planned_notes: Vec<PlannedNote>,
}

impl PlannedNotes {
    pub fn push(&mut self, planned_note: PlannedNote) {
        self.planned_notes.push(planned_note);
    }

    pub fn is_next_beat_message_pickup_to_the_last_note(&self, measure_beat: MeasureBeat) -> bool {
        let num_planned_notes = self.planned_notes.len();
        if num_planned_notes < 2 {
            return false;
        }
        let second_to_last_note = &self.planned_notes[num_planned_notes - 2];
        second_to_last_note.note_on.measure_beat == measure_beat
    }
}

impl Default for PlannedNotes {
    fn default() -> Self {
        Self {
            planned_notes: Vec::new(),
        }
    }
}

fn plan_initial_line(
    planned_notes: &mut PlannedNotes,
    line: &Line,
    outside_of_the_key_offset: i8,
    first_eligible_measure_beat: MeasureBeat,
) {
    let first_note_start_beat = line.notes[0].start;
    let mut current_measure = if first_note_start_beat.sixteenth_note
        >= first_eligible_measure_beat.beat_number.sixteenth_note
    {
        first_eligible_measure_beat.measure
    } else {
        first_eligible_measure_beat.measure + 1
    };
    for (note_index, note) in line.notes.iter().enumerate() {
        let did_measure_tick = note_index > 0
            && line.notes[note_index - 1].start.sixteenth_note > note.start.sixteenth_note;
        if did_measure_tick {
            current_measure += 1;
        }

        planned_notes.push(PlannedNote::new(
            note,
            MeasureBeat::new(note.start, current_measure),
            outside_of_the_key_offset,
        ));
    }
}

fn plan_overlapping_line(
    planned_notes: &mut PlannedNotes,
    line: &Line,
    outside_of_the_key_offset: i8,
    start_measure_beat: MeasureBeat,
) {
    planned_notes.planned_notes.pop();
    planned_notes.planned_notes.pop();

    let first_note_start_beat = line.notes[0].start;
    let sixteenth_notes_offset = start_measure_beat.beat_number.sixteenth_note as i8
        - first_note_start_beat.sixteenth_note as i8;
    let mut current_measure = start_measure_beat.measure;
    let mut last_adjusted_sixteenth_note: Option<u32> = None;
    for note in line.notes.iter() {
        let adjusted_sixteenth_note =
            (note.start.sixteenth_note as i8 + 16 + sixteenth_notes_offset) as u32 % 16;
        if let Some(last_adjusted_sixteenth_note) = last_adjusted_sixteenth_note {
            if last_adjusted_sixteenth_note > adjusted_sixteenth_note {
                current_measure += 1;
            }
        }

        planned_notes.push(PlannedNote::new(
            note,
            MeasureBeat::new(BeatNumber::new(adjusted_sixteenth_note), current_measure),
            outside_of_the_key_offset,
        ));

        last_adjusted_sixteenth_note = Some(adjusted_sixteenth_note);
    }
}

pub struct LineLauncher {
    lines: Vec<Line>,
    pub progression: Progression,
}

impl LineLauncher {
    pub fn from(progression: Progression) -> Self {
        Self {
            lines: Line::outside_of_the_key_lines(),
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
        randomize_note_start_time_ratio_slider: Option<MidiSlider>,
    ) {
        let midi_message_sender = MidiMessageSender::new(output);
        let mut planned_notes = PlannedNotes::default();
        let mut next_measure_beat = MeasureBeat::default();
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
        let (mut randomize_note_start_time_ratio, randomize_note_start_time_ratio_receiver) =
            match randomize_note_start_time_ratio_slider {
                Some(randomize_note_start_time_ratio_slider) => (
                    Some(0.0),
                    listen_for_control_changes(
                        midi_message_bus.add_rx(),
                        randomize_note_start_time_ratio_slider,
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
        let mut thread_rng = rand::thread_rng();

        let mut last_planned_line_index = thread_rng.gen_range(0..self.lines.len());
        let mut last_planned_outside_of_the_key_offset = thread_rng.gen_range(0..12);

        plan_initial_line(
            &mut planned_notes,
            &self.lines[last_planned_line_index],
            last_planned_outside_of_the_key_offset,
            next_measure_beat.incremented(),
        );
        debug!(
            "Planned line index {} and outside of the key offset {}:",
            last_planned_line_index, last_planned_outside_of_the_key_offset
        );
        for (planned_note_index, planned_note) in planned_notes.planned_notes.iter().enumerate() {
            debug!(
                "planned_note_index {} note on: {:?}",
                planned_note_index, planned_note.note_on
            );
            debug!(
                "planned_note_index {} note off: {:?}",
                planned_note_index, planned_note.note_off
            );
        }

        for message in get_combined_message_receiver(
            beat_message_receiver,
            duration_ratio_receiver,
            ahead_or_behind_the_beat_ratio_receiver,
            randomize_note_start_time_ratio_receiver,
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
                    self.possibly_schedule_note_on(
                        &planned_notes,
                        &next_measure_beat,
                        &schedule_note_on_sender,
                        &duration_between_sixteenth_notes,
                        &ahead_or_behind_the_beat_ratio,
                        &randomize_note_start_time_ratio,
                        &mut thread_rng,
                    );

                    if planned_notes.is_next_beat_message_pickup_to_the_last_note(
                        next_measure_beat.incremented(),
                    ) {
                        let (new_line_index, new_outside_of_the_key_offset) = self
                            .find_line_in_different_key(
                                &planned_notes,
                                last_planned_line_index,
                                last_planned_outside_of_the_key_offset,
                                &mut thread_rng,
                            );
                        last_planned_line_index = new_line_index;
                        last_planned_outside_of_the_key_offset = new_outside_of_the_key_offset;
                        plan_overlapping_line(
                            &mut planned_notes,
                            &self.lines[last_planned_line_index],
                            last_planned_outside_of_the_key_offset,
                            next_measure_beat.incremented(),
                        );
                        debug!(
                            "Planned overlapping line index {} and outside of the key offset {}:",
                            last_planned_line_index, last_planned_outside_of_the_key_offset
                        );
                        for (planned_note_index, planned_note) in
                            planned_notes.planned_notes.iter().enumerate()
                        {
                            debug!(
                                "planned_note_index {} note on: {:?}",
                                planned_note_index, planned_note.note_on
                            );
                            debug!(
                                "planned_note_index {} note off: {:?}",
                                planned_note_index, planned_note.note_off
                            );
                        }
                    }

                    next_measure_beat.increment();
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
                CombinedMessage::RandomizeNoteStartTimeRatioMessage(
                    new_randomize_note_start_time_ratio,
                ) => {
                    debug!(
                        "randomize note start time ratio change: {}",
                        new_randomize_note_start_time_ratio
                    );
                    randomize_note_start_time_ratio = Some(new_randomize_note_start_time_ratio);
                }
                CombinedMessage::FireNoteOffMessage(fire_note_off_message) => {
                    let planned_note =
                        &mut planned_notes.planned_notes[fire_note_off_message.planned_note_index];
                    if planned_note.has_note_off_fired {
                        debug!(
                            "Note off already fired for planned_note_index: {}",
                            fire_note_off_message.planned_note_index
                        );
                    } else {
                        debug!("Firing note off: {:?}", fire_note_off_message);
                        midi_message_sender.fire_note_off(planned_note.note_off.note);

                        planned_note.has_note_off_fired = true;
                    }
                }
                CombinedMessage::FireNoteOnMessage(fire_note_on_message) => {
                    debug!("Received fire note on message: {:?}", fire_note_on_message);
                    fire_preceding_note_off_if_unfired(
                        &mut planned_notes,
                        fire_note_on_message.planned_note_index,
                        &midi_message_sender,
                    );
                    let planned_note =
                        &mut planned_notes.planned_notes[fire_note_on_message.planned_note_index];
                    midi_message_sender.fire_note_on(planned_note.note_on.note);
                    planned_note.has_note_on_fired = true;
                    if let Some(duration_ratio) = duration_ratio {
                        if let Some(duration_between_sixteenth_notes) =
                            duration_between_sixteenth_notes.get_duration()
                        {
                            debug!(
                                "Sending schedule note off message, planned_note_index: {}, now: {:?}, duration_ratio: {}",
                                fire_note_on_message.planned_note_index, SystemTime::now(), duration_ratio
                            );
                            schedule_note_off_sender
                                .send(ScheduleNoteOffMessage {
                                    time: SystemTime::now()
                                        + duration_between_sixteenth_notes.mul_f64(duration_ratio),
                                    planned_note_index: fire_note_on_message.planned_note_index,
                                })
                                .unwrap();
                        }
                    }
                }
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn possibly_schedule_note_on(
        &self,
        planned_notes: &PlannedNotes,
        current_measure_beat: &MeasureBeat,
        schedule_note_on_sender: &Sender<ScheduleNoteOnMessage>,
        duration_between_sixteenth_notes: &DurationBetweenSixteenthNotes,
        ahead_or_behind_the_beat_ratio: &Option<f64>,
        randomize_note_start_time_ratio: &Option<f64>,
        thread_rng: &mut ThreadRng,
    ) {
        let note_to_schedule_index = planned_notes.planned_notes.iter().position(|planned_note| {
            planned_note.note_on.measure_beat == current_measure_beat.incremented()
        });
        let note_to_schedule_index = match note_to_schedule_index {
            Some(note_index) => note_index,
            None => return,
        };
        let note_to_schedule = &planned_notes.planned_notes[note_to_schedule_index];
        let duration_between_sixteenth_notes = match duration_between_sixteenth_notes.get_duration()
        {
            Some(duration_between_sixteenth_notes) => duration_between_sixteenth_notes,
            None => panic!("Couldn't schedule note on because no duration between sixteenth notes"),
        };
        debug!(
            "Sending schedule note on message, planned_note: {:?}, now: {:?}",
            note_to_schedule,
            SystemTime::now()
        );
        let random_ratio_of_duration_between_sixteenth_notes =
            if let Some(randomize_note_start_time_ratio) = randomize_note_start_time_ratio {
                (thread_rng.gen::<f64>() - 0.5) * randomize_note_start_time_ratio
            } else {
                0.0
            };
        schedule_note_on_sender
            .send(ScheduleNoteOnMessage {
                time: SystemTime::now()
                    + duration_between_sixteenth_notes.mul_f64(
                        (1.0 + ((0.5 - ahead_or_behind_the_beat_ratio.unwrap_or(0.5)) * 2.0)
                            + random_ratio_of_duration_between_sixteenth_notes)
                            .max(0.0),
                    ),
                planned_note_index: note_to_schedule_index,
            })
            .unwrap();
    }

    fn find_line_in_different_key(
        &self,
        planned_notes: &PlannedNotes,
        _last_planned_line_index: usize,
        _last_planned_outside_of_the_key_offset: i8,
        thread_rng: &mut ThreadRng,
    ) -> (usize, i8) {
        let last_two_planned_notes: (u8, u8) = (
            planned_notes.planned_notes[planned_notes.planned_notes.len() - 2]
                .note_on
                .note
                .into(),
            planned_notes.planned_notes[planned_notes.planned_notes.len() - 1]
                .note_on
                .note
                .into(),
        );
        loop {
            let new_outside_of_the_key_offset = thread_rng.gen_range(0..12);
            debug!(
                "Trying to find line for outside_of_the_key_offset {}",
                new_outside_of_the_key_offset
            );
            let mut octave_adjustment: i8 = 0;
            let new_line_index = self.lines.iter().position(|line| {
                let first_note_in_new_outside_key: u8 = line.notes[0]
                    .note
                    .step(new_outside_of_the_key_offset)
                    .unwrap()
                    .into();
                let (is_within_a_half_step, first_octave_adjustment) = get_is_within_a_half_step(
                    first_note_in_new_outside_key,
                    last_two_planned_notes.0,
                );
                if !is_within_a_half_step {
                    return false;
                }
                let second_note_in_new_outside_key: u8 = line.notes[1]
                    .note
                    .step(new_outside_of_the_key_offset)
                    .unwrap()
                    .into();
                let (is_within_a_half_step, second_octave_adjustment) = get_is_within_a_half_step(
                    second_note_in_new_outside_key,
                    last_two_planned_notes.1,
                );
                if !is_within_a_half_step {
                    return false;
                }
                if first_octave_adjustment != second_octave_adjustment {
                    return false;
                }
                octave_adjustment = first_octave_adjustment;
                true
            });
            if let Some(new_line_index) = new_line_index {
                let new_outside_of_the_key_offset_octave_adjusted =
                    new_outside_of_the_key_offset + octave_adjustment;
                debug!(
                    "found line index: {}, octave-adjusted outside_of_the_key_offset: {}",
                    new_line_index, new_outside_of_the_key_offset_octave_adjusted
                );
                return (
                    new_line_index,
                    new_outside_of_the_key_offset_octave_adjusted,
                );
            }
        }
    }
}

fn get_is_within_a_half_step(note_a: u8, note_b: u8) -> (bool, i8) {
    let raw_difference = note_a as i8 - note_b as i8;
    let difference_within_same_octave = raw_difference.rem_euclid(12);
    if difference_within_same_octave == 0 {
        return (true, -raw_difference);
    }
    if difference_within_same_octave == 1 {
        return (true, -(raw_difference - 1));
    }
    if difference_within_same_octave == 11 {
        return (true, -(raw_difference + 1));
    }
    (false, 0)
}

fn fire_preceding_note_off_if_unfired(
    planned_notes: &mut PlannedNotes,
    planned_note_index: usize,
    midi_message_sender: &MidiMessageSender,
) {
    if planned_note_index == 0 {
        return;
    }

    let previous_planned_note = &mut planned_notes.planned_notes[planned_note_index - 1];

    if previous_planned_note.has_note_off_fired {
        return;
    }

    debug!(
        "Synchronously firing note off: {:?}",
        previous_planned_note.note_off
    );
    midi_message_sender.fire_note_off(previous_planned_note.note_off.note);

    previous_planned_note.has_note_off_fired = true;
}
