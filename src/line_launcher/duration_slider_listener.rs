use crate::{config, config::midi::MidiSlider, midi, Message};
use std::sync::{mpsc::Receiver, Arc, Mutex};
use std::thread;
use wmidi::Channel;

pub fn listen_for_duration_control_changes(
    midi_messages_receiver: Receiver<Message>,
    value: Arc<Mutex<f64>>,
) {
    thread::spawn(move || {
        let slider = MidiSlider {
            channel: Channel::from_index(15).unwrap(),
            control_change: config::midi::u8_to_control_function(28).unwrap(),
        };

        for midi_message in midi_messages_receiver.iter() {
            let midi_messages = vec![midi_message];
            if let Some(new_value) = control_value_from_midi_messages(&midi_messages, slider) {
                let mut value = value.lock().unwrap();
                *value = get_percent(new_value);
            }
        }
    });
}

fn control_value_from_midi_messages(midi_messages: &[Message], slider: MidiSlider) -> Option<u32> {
    let control_value = midi::latest_control_value(slider, midi_messages)?;
    let new_value = midi::interpolate_control_value(0, 100, control_value);

    Some(new_value)
}

fn get_percent(value: u32) -> f64 {
    value as f64
}
