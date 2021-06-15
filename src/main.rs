use midir::os::unix::{VirtualInput, VirtualOutput};
use midir::{MidiInput, MidiOutput};

fn main() {
    let midi_out = MidiOutput::new("Line runner").unwrap();

    let /*mut*/ _conn_out = midi_out.create_virtual("Line runner").unwrap();

    let midi_in = MidiInput::new("Line runner").unwrap();

    let _conn_in = midi_in
        .create_virtual(
            "Line runner",
            |stamp, message, _| {
                println!("{}: {:?} (len = {})", stamp, message, message.len());
            },
            (),
        )
        .unwrap();

    loop {}
}
