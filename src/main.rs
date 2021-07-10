use midir::os::unix::{VirtualInput, VirtualOutput};
use midir::{MidiInput, MidiOutput};
use wmidi::MidiMessage;

use line_runner::{config, midi, LineLauncher, Message, MidiClockTracker, Progression, Result};

fn main() -> Result<()> {
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

    let midi_config = config::midi::Midi {
        port: Some("Launchkey Mini MK3 DAW Port".to_string()),
    };
    let midi_messages = match &midi_config.port {
        Some(port_name) => Some(midi::listen_for_input(port_name)?),
        None => None,
    };

    let line_launcher: LineLauncher = Progression::parse("C C C C Eb Eb Eb Eb").unwrap().into();
    line_launcher.listen(beat_message_receiver, conn_out, midi_messages.unwrap());

    Ok(())
}

fn handle_message(message: Message, midi_clock_tracker: &mut MidiClockTracker) {
    if message.message == MidiMessage::TimingClock {
        midi_clock_tracker.tick();
    }
}
