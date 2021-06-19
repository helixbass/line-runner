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
    vec![parse_line("C4 F3 G3 Bb3 C4 Db4 Eb4 F4 E4 . . .")]
}
