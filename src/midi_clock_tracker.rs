use std::sync::mpsc::{self, Receiver, Sender};

use crate::BeatNumber;

const TICKS_PER_QUARTER_NOTE: u32 = 24;

pub struct MidiClockTracker {
    ticks_received: u32,
    sender: Sender<BeatNumber>,
}

impl MidiClockTracker {
    pub fn new() -> (Self, Receiver<BeatNumber>) {
        let (sender, receiver) = mpsc::channel();

        (
            Self {
                ticks_received: 0,
                sender,
            },
            receiver,
        )
    }

    pub fn tick(&mut self) {
        self.ticks_received += 1;
        self.emit_beat_number();
    }

    fn emit_beat_number(&self) {
        let use_ticks_received = self.ticks_received - 1;

        if use_ticks_received % (TICKS_PER_QUARTER_NOTE / 4) != 0 {
            return;
        }

        let ticks_this_measure = use_ticks_received % (TICKS_PER_QUARTER_NOTE * 4);

        let quarter_note = (ticks_this_measure / TICKS_PER_QUARTER_NOTE) + 1;

        let ticks_this_quarter_note = ticks_this_measure % TICKS_PER_QUARTER_NOTE;

        let sixteenth_note = (ticks_this_quarter_note / (TICKS_PER_QUARTER_NOTE / 4)) + 1;

        self.sender
            .send(BeatNumber {
                quarter_note,
                sixteenth_note,
            })
            .unwrap();
    }
}
