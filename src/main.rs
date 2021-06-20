use midir::os::unix::{VirtualInput, VirtualOutput};
use midir::{MidiInput, MidiOutput};
use wmidi::MidiMessage;

use line_runner::{LineLauncher, Message, MidiClockTracker};

fn main() {
    let midi_out = MidiOutput::new("Line runner").unwrap();

    let conn_out = midi_out.create_virtual("Line runner").unwrap();

    let midi_in = MidiInput::new("Line runner").unwrap();

    let (mut midi_clock_tracker, beat_message_receiver) = MidiClockTracker::new();

    let _conn_in = midi_in
        .create_virtual(
            "Line runner",
            move |timestamp, bytes, _| {
                if let Some(message) = Message::from(timestamp, bytes).unwrap() {
                    handle_message(message, &mut midi_clock_tracker);
                }
            },
            (),
        )
        .unwrap();

    let line_launcher = LineLauncher::default();
    line_launcher.listen(beat_message_receiver, conn_out);
}

fn handle_message(message: Message, midi_clock_tracker: &mut MidiClockTracker) {
    if message.message == MidiMessage::TimingClock {
        midi_clock_tracker.tick();
    }
}
