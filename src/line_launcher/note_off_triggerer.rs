use std::sync::{
    mpsc::{self, Receiver, Sender},
    Arc, Mutex,
};
use std::thread::{self, JoinHandle};
use std::time::SystemTime;
use wmidi::Note;

use super::{MidiMessageSender, PlayingState};

pub struct NoteOffInstruction {
    pub note: Note,
    pub time: SystemTime,
    pub note_index: usize,
}

pub struct NoteOffTriggerer {
    receiver: Receiver<NoteOffInstruction>,
    midi_message_sender: MidiMessageSender,
    playing_state: Arc<Mutex<PlayingState>>,
}

impl NoteOffTriggerer {
    pub fn new(
        midi_message_sender: MidiMessageSender,
        playing_state: Arc<Mutex<PlayingState>>,
    ) -> (Self, Sender<NoteOffInstruction>) {
        let (sender, receiver) = mpsc::channel();

        let note_off_triggerer = Self {
            receiver,
            midi_message_sender,
            playing_state,
        };

        (note_off_triggerer, sender)
    }

    pub fn listen(self) -> JoinHandle<()> {
        thread::spawn(move || loop {
            let note_off_instruction = self.receiver.recv().unwrap();

            let now = SystemTime::now();
            let from_now = note_off_instruction
                .time
                .duration_since(now)
                .unwrap_or_default();

            spin_sleep::sleep(from_now);

            let mut playing_state = self.playing_state.lock().unwrap();

            match *playing_state {
                PlayingState::Playing {
                    next_note_off_index,
                    line_index,
                    next_note_index,
                    pitch_offset,
                } if next_note_off_index <= note_off_instruction.note_index => {
                    self.midi_message_sender
                        .fire_note_off(note_off_instruction.note);

                    *playing_state = PlayingState::Playing {
                        line_index,
                        next_note_index,
                        pitch_offset,
                        next_note_off_index: note_off_instruction.note_index,
                    };
                }
                _ => (),
            }
        })
    }
}
