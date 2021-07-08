use crate::Message;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};

pub struct MidiMessagePublisher {
    midi_messages: Receiver<Message>,
    senders: Vec<Sender<Message>>,
}

impl MidiMessagePublisher {
    pub fn new(midi_messages: Receiver<Message>) -> Self {
        Self {
            midi_messages,
            senders: Vec::new(),
        }
    }

    pub fn get_receiver(&mut self) -> Receiver<Message> {
        let (sender, receiver) = mpsc::channel::<Message>();
        self.senders.push(sender);
        receiver
    }

    pub fn listen(self) -> JoinHandle<()> {
        thread::spawn(move || {
            for message in self.midi_messages.iter() {
                for sender in &self.senders {
                    sender.send(message.clone()).unwrap();
                }
            }
        })
    }
}
