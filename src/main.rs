use midir::os::unix::{VirtualInput, VirtualOutput};
use midir::{MidiInput, MidiOutput};
use std::sync::{Arc, Mutex};
use wmidi::MidiMessage;

use line_runner::{LineLauncher, Message, MidiClockTracker};

fn main() {
    let midi_out = MidiOutput::new("Line runner").unwrap();

    let conn_out = midi_out.create_virtual("Line runner").unwrap();

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

    let mut line_launcher = LineLauncher::new(beat_message_receiver, conn_out);
    line_launcher.listen();
}

fn handle_message(message: Message, midi_clock_tracker: &Arc<Mutex<MidiClockTracker>>) -> () {
    match message.message {
        MidiMessage::TimingClock => {
            midi_clock_tracker.lock().unwrap().tick();
        }
        _ => {}
    }
}
