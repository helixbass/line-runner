use midir::os::unix::{VirtualInput, VirtualOutput};
use midir::{MidiInput, MidiOutput};
use wmidi::MidiMessage;

use line_runner::message::Message;

fn main() {
    let midi_out = MidiOutput::new("Line runner").unwrap();

    let /*mut*/ _conn_out = midi_out.create_virtual("Line runner").unwrap();

    let midi_in = MidiInput::new("Line runner").unwrap();

    let _conn_in = midi_in
        .create_virtual(
            "Line runner",
            |timestamp, bytes, _| {
                if let Some(message) = Message::from(timestamp, bytes).unwrap() {
                    handle_message(message);
                }
            },
            (),
        )
        .unwrap();

    loop {}
}

fn handle_message(message: Message) -> () {
    match message.message {
        MidiMessage::TimingClock => {
            println!("got timing clock message");
        }
        _ => {}
    }
}
