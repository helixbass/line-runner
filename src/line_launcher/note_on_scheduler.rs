use log::*;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};
use std::time::SystemTime;

#[derive(Debug)]
pub struct ScheduleNoteOnMessage {
    pub time: SystemTime,
    pub planned_note_index: usize,
}

#[derive(Debug)]
pub struct FireNoteOnMessage {
    pub planned_note_index: usize,
}

pub struct NoteOnScheduler {
    schedule_note_on_receiver: Receiver<ScheduleNoteOnMessage>,
    fire_note_on_sender: Sender<FireNoteOnMessage>,
}

impl NoteOnScheduler {
    pub fn new() -> (
        Self,
        Sender<ScheduleNoteOnMessage>,
        Receiver<FireNoteOnMessage>,
    ) {
        let (schedule_note_on_sender, schedule_note_on_receiver) = mpsc::channel();
        let (fire_note_on_sender, fire_note_on_receiver) = mpsc::channel();

        let note_on_triggerer = Self {
            schedule_note_on_receiver,
            fire_note_on_sender,
        };

        (
            note_on_triggerer,
            schedule_note_on_sender,
            fire_note_on_receiver,
        )
    }

    pub fn listen(self) -> JoinHandle<()> {
        thread::spawn(move || loop {
            let schedule_note_on_message = self.schedule_note_on_receiver.recv().unwrap();
            debug!(
                "Received schedule_note_on_message: {:?}",
                schedule_note_on_message,
            );

            let now = SystemTime::now();
            let from_now = schedule_note_on_message
                .time
                .duration_since(now)
                .unwrap_or_default();

            debug!("Sleeping, now: {:?}, from_now: {:?}", now, from_now);

            spin_sleep::sleep(from_now);

            debug!(
                "Sending fire note on, planned_note_index: {}, now: {:?}",
                schedule_note_on_message.planned_note_index,
                SystemTime::now()
            );
            self.fire_note_on_sender
                .send(FireNoteOnMessage {
                    planned_note_index: schedule_note_on_message.planned_note_index,
                })
                .unwrap();
        })
    }
}
