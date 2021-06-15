use midir::os::unix::{VirtualInput, VirtualOutput};
use midir::{MidiInput, MidiOutput};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use wmidi::MidiMessage;

use line_runner::message::Message;

fn main() {
    let midi_out = MidiOutput::new("Line runner").unwrap();

    let /*mut*/ _conn_out = midi_out.create_virtual("Line runner").unwrap();

    let midi_in = MidiInput::new("Line runner").unwrap();

    let (midi_clock_tracker, beat_message_receiver) = MidiClockTracker::new();
    let midi_clock_tracker = Arc::new(Mutex::new(midi_clock_tracker));
    let midi_clock_tracker_clone = midi_clock_tracker.clone();

    let _conn_in = midi_in
        .create_virtual(
            "Line runner",
            move |timestamp, bytes, _| {
                if let Some(message) = Message::from(timestamp, bytes).unwrap() {
                    handle_message(message, &midi_clock_tracker_clone);
                }
            },
            (),
        )
        .unwrap();

    let line_launcher = LineLauncher::new(beat_message_receiver);
    line_launcher.listen();
}

struct LineLauncher {
    beat_message_receiver: Receiver<BeatNumber>,
}

impl LineLauncher {
    pub fn new(beat_message_receiver: Receiver<BeatNumber>) -> Self {
        Self {
            beat_message_receiver,
        }
    }

    pub fn listen(&self) -> () {
        loop {
            let beat_message = self.beat_message_receiver.recv().unwrap();
            println!(
                "Received beat message, quarter note: {}, sixteenth note: {}",
                beat_message.quarter_note, beat_message.sixteenth_note
            );
        }
    }
}

fn handle_message(message: Message, midi_clock_tracker: &Arc<Mutex<MidiClockTracker>>) -> () {
    match message.message {
        MidiMessage::TimingClock => {
            midi_clock_tracker.lock().unwrap().tick();
        }
        _ => {}
    }
}

struct BeatNumber {
    quarter_note: u32,
    sixteenth_note: u32,
}

impl BeatNumber {
    pub fn new(quarter_note: u32, sixteenth_note: u32) -> Self {
        Self {
            quarter_note,
            sixteenth_note,
        }
    }
}

const TICKS_PER_QUARTER_NOTE: u32 = 24;

// enum MidiClockTrackerMessage {
//     BeatNumber(BeatNumber),
// }

struct MidiClockTracker {
    ticks_received: u32,
    sender: Sender<BeatNumber>,
}

impl MidiClockTracker {
    pub fn new() -> (Self, Receiver<BeatNumber>) {
        let (sender, receiver) = mpsc::channel();

        (
            Self {
                ticks_received: 0,
                sender,
            },
            receiver,
        )
    }

    pub fn tick(&mut self) -> () {
        self.ticks_received += 1;
        self.emit_beat_number();
    }

    fn emit_beat_number(&self) -> () {
        let use_ticks_received = self.ticks_received - 1;

        if use_ticks_received % (TICKS_PER_QUARTER_NOTE / 4) != 0 {
            return;
        }

        let ticks_this_measure = use_ticks_received % (TICKS_PER_QUARTER_NOTE * 4);

        let quarter_note = (ticks_this_measure / TICKS_PER_QUARTER_NOTE) + 1;

        let ticks_this_quarter_note = ticks_this_measure % TICKS_PER_QUARTER_NOTE;

        let sixteenth_note = (ticks_this_quarter_note / (TICKS_PER_QUARTER_NOTE / 4)) + 1;

        self.sender
            .send(BeatNumber::new(quarter_note, sixteenth_note))
            .unwrap();
    }
}
