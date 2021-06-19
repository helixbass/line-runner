use wmidi::Note;

use crate::{parse_line, BeatNumber};

#[derive(PartialEq, Debug)]
pub struct LineNote {
    pub start: BeatNumber,
    pub duration: u32,
    pub note: Note,
}

#[derive(PartialEq, Debug)]
pub struct Line {
    pub notes: Vec<LineNote>,
}

impl Line {
    pub fn new(notes: Vec<LineNote>) -> Self {
        Self { notes }
    }
}

pub fn all_lines() -> Vec<Line> {
    vec![parse_line("C5 F4 G4 Bb4 C5 Db5 Eb5 F5 E5 . . .")]
}
