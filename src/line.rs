use wmidi::Note;

use crate::BeatNumber;

pub struct LineNote {
    pub start: BeatNumber,
    pub duration: u32,
    pub note: Note,
}

pub struct Line {
    pub notes: Vec<LineNote>,
}

impl Line {
    pub fn new(notes: Vec<LineNote>) -> Self {
        Self { notes }
    }
}

pub fn all_lines() -> Vec<Line> {
    vec![Line::new(vec![
        LineNote {
            start: BeatNumber {
                quarter_note: 1,
                sixteenth_note: 1,
            },
            duration: 1,
            note: Note::C4,
        },
        LineNote {
            start: BeatNumber {
                quarter_note: 1,
                sixteenth_note: 2,
            },
            duration: 1,
            note: Note::F3,
        },
        LineNote {
            start: BeatNumber {
                quarter_note: 1,
                sixteenth_note: 3,
            },
            duration: 1,
            note: Note::G3,
        },
        LineNote {
            start: BeatNumber {
                quarter_note: 1,
                sixteenth_note: 4,
            },
            duration: 1,
            note: Note::Bb3,
        },
        LineNote {
            start: BeatNumber {
                quarter_note: 2,
                sixteenth_note: 1,
            },
            duration: 1,
            note: Note::C4,
        },
        LineNote {
            start: BeatNumber {
                quarter_note: 2,
                sixteenth_note: 2,
            },
            duration: 1,
            note: Note::Db4,
        },
        LineNote {
            start: BeatNumber {
                quarter_note: 2,
                sixteenth_note: 3,
            },
            duration: 1,
            note: Note::Eb4,
        },
        LineNote {
            start: BeatNumber {
                quarter_note: 2,
                sixteenth_note: 4,
            },
            duration: 1,
            note: Note::F4,
        },
        LineNote {
            start: BeatNumber {
                quarter_note: 3,
                sixteenth_note: 1,
            },
            duration: 4,
            note: Note::E4,
        },
    ])]
}
