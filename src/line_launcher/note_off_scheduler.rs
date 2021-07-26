use log::*;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};
use std::time::SystemTime;

#[derive(Debug)]
pub struct ScheduleNoteOffMessage {
    pub time: SystemTime,
    pub note_index: usize,
}

#[derive(Debug)]
pub struct FireNoteOffMessage {
    pub note_index: usize,
}

pub struct NoteOffScheduler {
    schedule_note_off_receiver: Receiver<ScheduleNoteOffMessage>,
    fire_note_off_sender: Sender<FireNoteOffMessage>,
}

impl NoteOffScheduler {
    pub fn new() -> (
        Self,
        Sender<ScheduleNoteOffMessage>,
        Receiver<FireNoteOffMessage>,
    ) {
        let (schedule_note_off_sender, schedule_note_off_receiver) = mpsc::channel();
        let (fire_note_off_sender, fire_note_off_receiver) = mpsc::channel();

        let note_off_scheduler = Self {
            schedule_note_off_receiver,
            fire_note_off_sender,
        };

        (
            note_off_scheduler,
            schedule_note_off_sender,
            fire_note_off_receiver,
        )
    }

    pub fn listen(self) -> JoinHandle<()> {
        thread::spawn(move || loop {
            let schedule_note_off_message = self.schedule_note_off_receiver.recv().unwrap();
            debug!(
                "Received schedule_note_off_message: {:?}",
                schedule_note_off_message,
            );

            let now = SystemTime::now();
            let from_now = schedule_note_off_message
                .time
                .duration_since(now)
                .unwrap_or_default();

            spin_sleep::sleep(from_now);

            debug!(
                "Sending fire note off, note_index: {}",
                schedule_note_off_message.note_index,
            );
            self.fire_note_off_sender
                .send(FireNoteOffMessage {
                    note_index: schedule_note_off_message.note_index,
                })
                .unwrap();
        })
    }
}
