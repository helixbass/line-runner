use log::*;
use midir::os::unix::{VirtualInput, VirtualOutput};
use midir::{MidiInput, MidiOutput};
use std::env;
use std::fs;
use wmidi::MidiMessage;

use line_runner::{config, midi, Config, LineLauncher, Message, MidiClockTracker, Result};

fn main() -> Result<()> {
    stderrlog::new()
        .module(module_path!())
        .verbosity(2)
        .init()
        .unwrap();

    let config = get_config()?;

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

    let midi_port_names = midi::port_names()?;

    if config.midi.port.is_none() && !midi_port_names.is_empty() {
        warn!(
            "Config is missing 'midi.port'. Available MIDI ports are:\n{}",
            midi_port_names.join("\n")
        );
    }

    let midi_messages = match &config.midi.port {
        Some(port_name) => Some(midi::listen_for_input(port_name)?),
        None => None,
    };

    let Config {
        progression,
        midi: config::midi::Midi {
            duration_ratio_slider,
            ..
        },
        ..
    } = config;
    let line_launcher = LineLauncher::from(progression);
    line_launcher.listen(
        beat_message_receiver,
        conn_out,
        midi_messages,
        duration_ratio_slider,
    );

    Ok(())
}

fn handle_message(message: Message, midi_clock_tracker: &mut MidiClockTracker) {
    if message.message == MidiMessage::TimingClock {
        midi_clock_tracker.tick();
    }
}

fn get_config() -> Result<Config> {
    let config = env::args()
        .nth(1)
        .map(|path| config_from_path(&path))
        .transpose()?;
    Ok(config.unwrap_or_default())
}

fn config_from_path(path: &str) -> Result<Config> {
    info!("Reading config from {}", path);

    let contents = fs::read_to_string(path)?;
    Config::from(&contents)
}
