use midir::os::unix::{VirtualInput, VirtualOutput};
use midir::{MidiInput, MidiOutput};
use std::sync::{Arc, Mutex};
use wmidi::MidiMessage;

use line_runner::message::Message;

fn main() {
    let midi_out = MidiOutput::new("Line runner").unwrap();

    let /*mut*/ _conn_out = midi_out.create_virtual("Line runner").unwrap();

    let midi_in = MidiInput::new("Line runner").unwrap();

    let midi_clock_tracker = Arc::new(Mutex::new(MidiClockTracker::new()));
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

    loop {}
}

fn handle_message(message: Message, midi_clock_tracker: &Arc<Mutex<MidiClockTracker>>) -> () {
    match message.message {
        MidiMessage::TimingClock => {
            println!("got timing clock message");
            midi_clock_tracker.lock().unwrap().tick();
        }
        _ => {}
    }
}

struct MidiClockTracker {
    ticks_received: u32,
}

impl MidiClockTracker {
    pub fn new() -> Self {
        Self { ticks_received: 0 }
    }

    pub fn tick(&mut self) -> () {
        self.ticks_received += 1;
    }
}
