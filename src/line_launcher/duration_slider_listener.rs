use crate::{config, config::midi::MidiSlider, midi, Message};
use bus::BusReader;
use std::sync::mpsc::{self, Receiver};
use std::thread;
use wmidi::Channel;

pub fn listen_for_duration_control_changes(
    mut midi_messages_receiver: BusReader<Message>,
) -> Receiver<f64> {
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || {
        let slider = MidiSlider {
            channel: Channel::from_index(15).unwrap(),
            control_change: config::midi::u8_to_control_function(28).unwrap(),
        };

        for midi_message in midi_messages_receiver.iter() {
            if let Some(new_value) = control_value_ratio_from_midi_message(&midi_message, slider) {
                sender.send(new_value).unwrap();
            }
        }
    });
    receiver
}

fn control_value_ratio_from_midi_message(
    midi_message: &Message,
    slider: MidiSlider,
) -> Option<f64> {
    let control_value = midi::get_control_value(slider, midi_message)?;
    let new_value = midi::interpolate_control_value(0.0, 1.0, control_value);

    Some(new_value)
}
