use std::sync::mpsc::Receiver;

use crate::BeatNumber;

pub struct LineLauncher {
    beat_message_receiver: Receiver<BeatNumber>,
}

impl LineLauncher {
    pub fn new(beat_message_receiver: Receiver<BeatNumber>) -> Self {
        Self {
            beat_message_receiver,
        }
    }

    pub fn listen(&self) -> () {
        let mut has_launched = false;
        loop {
            let beat_message = self.beat_message_receiver.recv().unwrap();
            println!(
                "Received beat message, quarter note: {}, sixteenth note: {}",
                beat_message.quarter_note, beat_message.sixteenth_note
            );
            if beat_message.quarter_note == 1 && beat_message.sixteenth_note == 1 && !has_launched {
                has_launched = true;
                println!("launching");
            }
        }
    }
}
