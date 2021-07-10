use crate::{config, config::midi::MidiSlider, midi, Message};
use bus::BusReader;
use std::sync::{Arc, Mutex};
use std::thread;
use wmidi::Channel;

pub fn listen_for_duration_control_changes(
    mut midi_messages_receiver: BusReader<Message>,
    value: Arc<Mutex<f64>>,
) {
    thread::spawn(move || {
        let slider = MidiSlider {
            channel: Channel::from_index(15).unwrap(),
            control_change: config::midi::u8_to_control_function(28).unwrap(),
        };

        for midi_message in midi_messages_receiver.iter() {
            let midi_messages = vec![midi_message];
            if let Some(new_value) = control_value_ratio_from_midi_messages(&midi_messages, slider)
            {
                let mut value = value.lock().unwrap();
                *value = new_value;
            }
        }
    });
}

fn control_value_ratio_from_midi_messages(
    midi_messages: &[Message],
    slider: MidiSlider,
) -> Option<f64> {
    let control_value = midi::latest_control_value(slider, midi_messages)?;
    let new_value = midi::interpolate_control_value(0.0, 1.0, control_value);

    Some(new_value)
}
