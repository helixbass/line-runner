use midir::os::unix::VirtualOutput;
use midir::MidiOutput;

fn main() {
    let midi_out = MidiOutput::new("Line runner").unwrap();

    let /*mut*/ _conn_out = midi_out.create_virtual("Line runner").unwrap();

    loop {}
}
