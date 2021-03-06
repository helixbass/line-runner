use wmidi::Note;

use crate::{BeatNumber, Result};

mod parser;

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

    pub fn all() -> Vec<Line> {
        vec![
            "C4 F3 G3 Bb3 C4 Db4 Eb4 F4 E4 . . .",
            "- Db4 Bb3 Db4 C4 . Bb3 G3 F3 Bb3 F3 Gb3 G3 Gb3 F3 G3 E3 . . .",
            "C4 F3 G3 Bb3 C4 Db4 Bb3 Db4 C4 . .",
        ]
        .into_iter()
        .map(Self::parse)
        .collect::<Result<Vec<_>>>()
        .unwrap()
    }
}
