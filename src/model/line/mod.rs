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
            "G4 Gb4 G4 A4 Bb4 C5 A4 Ab4 G4 C4 C5 .",
            "G4 Gb4 G4 A4 Bb4 C5 A4 Ab4 G4",
            "G4 Gb4 Ab4 Gb4 G4 Gb4 F4 E4 Eb4 .",
            "F4 . E4 . Eb4 E4 F4 Eb4 E4 C4 Bb3 G3 Bb3 C4 E4 F4 G4 .",
            "F4 . E4 . Eb4 E4 F4 Eb4 E4 Eb4 C4 .",
            "E4 Eb4 F4 Eb4 E4 Eb4 D4 Db4 C4 .",
            "E4 Eb4 F4 Eb4 E4 Eb4 D4 Db4 C4 C5 .",
            "C4 C5 Bb4 Db5 C5 Bb4 G4 F4 G4 .",
            "C4 C5 Bb4 Db5 C5 Bb4 G4 Gb4 Ab4 Gb4 G4 Gb4 F4 G4 E4 C4",
            "E4 C4 E4 F4 Gb4 Ab4 G4 F4 E4 .",
            "C4 B3 Db4 B3 C4 B3 Bb3 A3 Ab3 G3 Gb3 Ab3 G3 Gb3 F3 E3 Eb3 .",
            "C4 B3 Db4 B3 C4 B3 Bb3 A3 Ab3 G3 Gb3 Ab3 G3 Gb3 F3 G3 E3 .",
            "E4 Eb4 C4 G3 Bb3 Db4 B3 Db4 C4 .",
        ]
        .into_iter()
        .map(Self::parse)
        .collect::<Result<Vec<_>>>()
        .unwrap()
    }

    pub fn outside_of_the_key_lines() -> Vec<Line> {
        vec![
            "- - - A4 G4 E4 C4 E4 D4",
            "- - - F4 E4 C4 Bb3 C4 E4 G4 Bb4 D5 C5",
            "- - - F4 E4 C4 Bb3 C4 E4 G4 Bb4 D5 C5 Bb4 G4 Ab4 A4",
            "- - - Eb4 E4 G4 Bb4 G4 D5 C5 Bb4 A4 G4",
            "- - - D5 C5 G4 E4 Bb4 G4 F4 D4 Eb4 E4",
            "- - - D5 C5 G4 E4 Bb4 G4 F4 D4 F4 E4",
            "- - - Bb3 C4 Eb4 G4 Bb4 D5 Eb5 C5 Bb4 A4",
            "- - - Bb3 C4 Eb4 Gb4 G4 A4 Bb4 Gb4 A4 G4",
            "- - - Bb3 C4 Eb4 Gb4 G4 A4 Bb4 Gb4 A4 G4 Gb4 F4 E4 Eb4",
            "- - - D4 C4 A4 G4 D5 C5 A5 G5 F5 E5",
            "- - - D5 C5 Bb4 A4 Ab4 G4 Gb4 F4 Eb4 E4",
            "- - - D5 C5 Bb4 A4 Ab4 G4 Gb4 F4 G4 E4",
            "- - - E4 C4 G3 E4 F4 G4 A4 Bb4 A4 G4",
            "- - - E4 C4 G3 E4 F4 G4 A4 Bb4 A4 G4 A4 Bb4 D5 C5",
            "- - - E4 C4 G3 E4 F4 G4 A4 Bb4 A4 G4 Gb4 F4 G4 E4",
            "- - - E4 C4 G3 E4 F4 G4 A4 Bb4 A4 G4 Gb4 F4 Eb4 E4",
            "- - - E5 C5 G4 F4 Eb4 E4",
            "- - - E5 C5 G4 F4 Eb4 E4 C5 Bb4 A4 G4",
            "- - - E5 C5 G4 F4 Eb4 E4 C5 Bb4 D5 C5",
            "- - - E5 C5 G4 F4 Eb4 E4 C5 Bb4 D5 C5 Bb4 G4 F4 G4",
            "- - - Eb4 E4 C5 Bb4 D5 C5",
            "- - - Eb4 E4 C5 Bb4 D5 C5 Bb4 A4 Ab4 G4",
        ]
        .into_iter()
        .map(Self::parse)
        .collect::<Result<Vec<_>>>()
        .unwrap()
    }
}
