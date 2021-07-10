pub mod message;
mod midi_message_publisher;

pub use midi_message_publisher::MidiMessagePublisher;

use crate::{config::midi::MidiSlider, Message, Result};
use anyhow::anyhow;
use midir::{MidiInput, MidiInputPort};
use num_traits::Num;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Duration;
use wmidi::{ControlValue, MidiMessage};

pub fn listen_for_input(port_name: &str) -> Result<Receiver<Message>> {
    let port = port(port_name)?;

    println!("MIDI input: {}", port_name);

    let (sender, receiver) = mpsc::channel();
    handle_messages(port, sender);

    Ok(receiver)
}

fn port_names() -> Result<Vec<String>> {
    let midi_input = midi_input()?;
    midi_input
        .ports()
        .iter()
        .map(|port| midi_input.port_name(port).map_err(|err| err.into()))
        .collect()
}

pub fn latest_control_value(slider: MidiSlider, messages: &[Message]) -> Option<ControlValue> {
    messages
        .iter()
        .rev()
        .find_map(|message| match message.message {
            MidiMessage::ControlChange(channel, function, value)
                if channel == slider.channel && function == slider.control_change =>
            {
                Some(value)
            }
            _ => None,
        })
}

pub fn interpolate_control_value<TValue: Num + From<u8> + Copy>(
    min: TValue,
    max: TValue,
    value: ControlValue,
) -> TValue {
    let control_value_min: TValue = from_control_value(ControlValue::MIN);
    let control_value_max: TValue = from_control_value(ControlValue::MAX);

    let value: TValue = from_control_value(value);
    (value - control_value_min) * (max - min) / (control_value_max - control_value_min) + min
}

fn midi_input() -> Result<MidiInput> {
    Ok(MidiInput::new("Input")?)
}

fn port(name: &str) -> Result<MidiInputPort> {
    let names = port_names()?;
    let midi_input = midi_input()?;

    midi_input
        .ports()
        .into_iter()
        .find(|port| midi_input.port_name(&port) == Ok(name.into()))
        .ok_or_else(|| {
            anyhow!(
                "Could not find a MIDI port with name '{}'. Available ports are:\n{}",
                name,
                names.join("\n")
            )
        })
}

fn handle_messages(port: MidiInputPort, sender: Sender<Message>) {
    thread::spawn(move || {
        let _connection = midi_input()
            .unwrap()
            .connect(
                &port,
                "midir-read-input",
                move |timestamp, bytes, _| {
                    if let Some(message) = Message::from(timestamp, bytes).unwrap() {
                        sender.send(message).unwrap();
                    }
                },
                (),
            )
            .unwrap();

        thread::sleep(Duration::from_micros(u64::MAX));
    });
}

fn from_control_value<TValue: From<u8>>(value: ControlValue) -> TValue {
    let byte: u8 = value.into();
    byte.into()
}
