use wmidi::Note;

use crate::{BeatNumber, Line, LineNote};

#[derive(Debug)]
enum State {
    AwaitingNextToken,
    ParsedNoteLetter { note_letter: char },
    ParsedNoteName { note_name: String },
}

pub fn parse_line(line_str: &str) -> Line {
    LineParser::new(line_str).parse()
}

struct LineParser {
    chars: Vec<char>,
    next_index: usize,
    state: State,
    currently_sustained_note: Option<(Note, BeatNumber)>,
    current_beat_number: BeatNumber,
}

impl LineParser {
    pub fn new(line_str: &str) -> Self {
        Self {
            chars: line_str.chars().collect(),
            next_index: 0,
            state: State::AwaitingNextToken,
            currently_sustained_note: None,
            current_beat_number: BeatNumber { sixteenth_note: 0 },
        }
    }

    pub fn parse(&mut self) -> Line {
        let mut line_notes: Vec<LineNote> = vec![];

        while self.next_index < self.chars.len() {
            let next_char = self.chars[self.next_index];
            match (next_char, &self.state) {
                (' ', State::AwaitingNextToken) => {
                    self.forward(1);
                }
                ('C' | 'D' | 'E' | 'F' | 'G' | 'A' | 'B', State::AwaitingNextToken) => {
                    self.finish_note_if_sustaining(&mut line_notes);
                    self.state = State::ParsedNoteLetter {
                        note_letter: next_char,
                    };
                    self.forward(1);
                }
                ('b', State::ParsedNoteLetter { note_letter }) => {
                    self.state = State::ParsedNoteName {
                        note_name: vec![note_letter, &next_char].into_iter().collect(),
                    };
                    self.forward(1);
                }
                (
                    // TODO: handle -1
                    '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9',
                    State::ParsedNoteLetter { note_letter },
                ) => {
                    let note_letter = note_letter.to_string();
                    self.mark_sustaining_note(note_letter, next_char);
                    self.finished_parsing_beat();
                    self.forward(1);
                }
                (
                    '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9',
                    State::ParsedNoteName { note_name },
                ) => {
                    let note_name = note_name.to_string();
                    self.mark_sustaining_note(note_name, next_char);
                    self.finished_parsing_beat();
                    self.forward(1);
                }
                ('.', State::AwaitingNextToken) => {
                    self.finished_parsing_beat();
                    self.forward(1);
                }
                _ => {
                    panic!("Couldn't parse line string: {}", self.line_str());
                }
            }
        }

        match &self.state {
            State::AwaitingNextToken => {
                self.finish_note_if_sustaining(&mut line_notes);
            }
            _ => {
                panic!(
                    "Ended with half-parsed token {:?} when parsing line {}",
                    self.state,
                    self.line_str()
                );
            }
        }

        Line::new(line_notes)
    }

    fn forward(&mut self, num_chars: usize) {
        self.next_index += num_chars;
    }

    fn get_note(&self, note_name: &str, octave: i8) -> Note {
        match (note_name, octave) {
            ("C", 3) => Note::C3,
            ("Db", 3) => Note::Db3,
            ("D", 3) => Note::D3,
            ("Eb", 3) => Note::Eb3,
            ("E", 3) => Note::E3,
            ("F", 3) => Note::F3,
            ("Gb", 3) => Note::Gb3,
            ("G", 3) => Note::G3,
            ("Ab", 3) => Note::Ab3,
            ("A", 3) => Note::A3,
            ("Bb", 3) => Note::Bb3,
            ("B", 3) => Note::B3,
            ("C", 4) => Note::C4,
            ("Db", 4) => Note::Db4,
            ("D", 4) => Note::D4,
            ("Eb", 4) => Note::Eb4,
            ("E", 4) => Note::E4,
            ("F", 4) => Note::F4,
            ("Gb", 4) => Note::Gb4,
            ("G", 4) => Note::G4,
            ("Ab", 4) => Note::Ab4,
            ("A", 4) => Note::A4,
            ("Bb", 4) => Note::Bb4,
            ("B", 4) => Note::B4,
            ("C", 5) => Note::C5,
            ("Db", 5) => Note::Db5,
            ("D", 5) => Note::D5,
            ("Eb", 5) => Note::Eb5,
            ("E", 5) => Note::E5,
            ("F", 5) => Note::F5,
            ("Gb", 5) => Note::Gb5,
            ("G", 5) => Note::G5,
            ("Ab", 5) => Note::Ab5,
            ("A", 5) => Note::A5,
            ("Bb", 5) => Note::Bb5,
            ("B", 5) => Note::B5,
            ("C", 6) => Note::C6,
            ("Db", 6) => Note::Db6,
            ("D", 6) => Note::D6,
            ("Eb", 6) => Note::Eb6,
            ("E", 6) => Note::E6,
            ("F", 6) => Note::F6,
            ("Gb", 6) => Note::Gb6,
            ("G", 6) => Note::G6,
            ("Ab", 6) => Note::Ab6,
            ("A", 6) => Note::A6,
            ("Bb", 6) => Note::Bb6,
            ("B", 6) => Note::B6,
            _ => {
                panic!("Invalid note: {}{}", note_name, octave);
            }
        }
    }

    fn finished_parsing_beat(&mut self) {
        self.current_beat_number = self.current_beat_number.add_sixteenths(1);
        self.state = State::AwaitingNextToken;
    }

    fn mark_sustaining_note(&mut self, note_name: String, octave_char: char) {
        let octave = octave_char.to_digit(10).unwrap() as i8;
        self.currently_sustained_note =
            Some((self.get_note(&note_name, octave), self.current_beat_number));
    }

    fn line_str(&self) -> String {
        self.chars.iter().collect()
    }

    fn finish_note_if_sustaining(&self, line_notes: &mut Vec<LineNote>) {
        if let Some((note, beat_number)) = self.currently_sustained_note {
            line_notes.push(LineNote {
                note,
                start: beat_number,
                duration: self.current_beat_number.duration_since(&beat_number),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_parses_line_starting_on_downbeat() {
        assert_eq!(
            parse_line("C4 F3 G3 Bb3 C4 Db4 Eb4 F4 E4"),
            Line::new(vec![
                LineNote {
                    start: BeatNumber { sixteenth_note: 0 },
                    duration: 1,
                    note: Note::C4,
                },
                LineNote {
                    start: BeatNumber { sixteenth_note: 1 },
                    duration: 1,
                    note: Note::F3,
                },
                LineNote {
                    start: BeatNumber { sixteenth_note: 2 },
                    duration: 1,
                    note: Note::G3,
                },
                LineNote {
                    start: BeatNumber { sixteenth_note: 3 },
                    duration: 1,
                    note: Note::Bb3,
                },
                LineNote {
                    start: BeatNumber { sixteenth_note: 4 },
                    duration: 1,
                    note: Note::C4,
                },
                LineNote {
                    start: BeatNumber { sixteenth_note: 5 },
                    duration: 1,
                    note: Note::Db4,
                },
                LineNote {
                    start: BeatNumber { sixteenth_note: 6 },
                    duration: 1,
                    note: Note::Eb4,
                },
                LineNote {
                    start: BeatNumber { sixteenth_note: 7 },
                    duration: 1,
                    note: Note::F4,
                },
                LineNote {
                    start: BeatNumber { sixteenth_note: 8 },
                    duration: 1,
                    note: Note::E4,
                },
            ])
        )
    }

    #[test]
    fn it_parses_sustain() {
        assert_eq!(
            parse_line("C4 F3 G3 Bb3 C4 Db4 Eb4 F4 E4 . . ."),
            Line::new(vec![
                LineNote {
                    start: BeatNumber { sixteenth_note: 0 },
                    duration: 1,
                    note: Note::C4,
                },
                LineNote {
                    start: BeatNumber { sixteenth_note: 1 },
                    duration: 1,
                    note: Note::F3,
                },
                LineNote {
                    start: BeatNumber { sixteenth_note: 2 },
                    duration: 1,
                    note: Note::G3,
                },
                LineNote {
                    start: BeatNumber { sixteenth_note: 3 },
                    duration: 1,
                    note: Note::Bb3,
                },
                LineNote {
                    start: BeatNumber { sixteenth_note: 4 },
                    duration: 1,
                    note: Note::C4,
                },
                LineNote {
                    start: BeatNumber { sixteenth_note: 5 },
                    duration: 1,
                    note: Note::Db4,
                },
                LineNote {
                    start: BeatNumber { sixteenth_note: 6 },
                    duration: 1,
                    note: Note::Eb4,
                },
                LineNote {
                    start: BeatNumber { sixteenth_note: 7 },
                    duration: 1,
                    note: Note::F4,
                },
                LineNote {
                    start: BeatNumber { sixteenth_note: 8 },
                    duration: 4,
                    note: Note::E4,
                },
            ])
        )
    }
}
