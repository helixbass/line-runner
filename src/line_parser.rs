use crate::{BeatNumber, Line, LineNote, Result};
use combine::{
    choice, many, many1, optional,
    parser::char::{digit, spaces},
    token, Parser,
};

#[derive(Clone, Copy, Debug)]
enum Letter {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
}

#[derive(Clone, Copy, Debug)]
enum Modifier {
    Flat,
    Natural,
}

#[derive(Clone, Copy, Debug)]
struct Note {
    letter: Letter,
    modifier: Modifier,
    octave: i8,
    duration: u32,
}

#[derive(Clone, Copy, Debug)]
enum Value {
    Note(Note),
    Rest,
}

impl Note {
    fn to_wmidi_note(self) -> wmidi::Note {
        let letter_value = match self.letter {
            Letter::C => 0,
            Letter::D => 2,
            Letter::E => 4,
            Letter::F => 5,
            Letter::G => 7,
            Letter::A => 9,
            Letter::B => 11,
        };

        let modifier_value = match self.modifier {
            Modifier::Flat => -1,
            Modifier::Natural => 0,
        };

        let octave_value = self.octave + 1;

        let value = octave_value * 12 + letter_value + modifier_value;
        wmidi::Note::from_u8_lossy(value as u8)
    }
}

pub fn parse_line(line: &str) -> Result<Line> {
    let letter_parser = choice((
        token('A').map(|_| Letter::A),
        token('B').map(|_| Letter::B),
        token('C').map(|_| Letter::C),
        token('D').map(|_| Letter::D),
        token('E').map(|_| Letter::E),
        token('F').map(|_| Letter::F),
        token('G').map(|_| Letter::G),
    ));

    let flat_parser = token('b').map(|_| Modifier::Flat);

    let modifier_parser =
        optional(flat_parser).map(|modifier| modifier.unwrap_or(Modifier::Natural));

    let octave_parser = (optional(token('-')), digit()).map(|(negative, digit)| {
        digit.to_string().parse::<i8>().unwrap() * negative.map_or(1, |_| -1)
    });

    let pitch_parser = (letter_parser, modifier_parser, octave_parser, spaces())
        .map(|(letter, modifier, octave, _)| (letter, modifier, octave));

    let dot_parser = (token('.'), spaces()).map(|_| ());

    let duration_parser = many(dot_parser).map(|dots: Vec<_>| (dots.len() + 1) as u32);

    let note_parser =
        (pitch_parser, duration_parser).map(|((letter, modifier, octave), duration)| {
            Value::Note(Note {
                letter,
                modifier,
                octave,
                duration,
            })
        });

    let rest_parser = (token('-'), spaces()).map(|_| Value::Rest);

    let value_parser = choice((note_parser, rest_parser));

    let mut line_parser = many1(value_parser).map(|notes: Vec<_>| to_line(&notes));

    let (result, _) = line_parser.parse(line)?;

    Ok(result)
}

fn to_line(notes: &[Value]) -> Line {
    let mut line_notes = vec![];
    let mut start = BeatNumber { sixteenth_note: 0 };

    for note in notes {
        match note {
            Value::Note(note) => {
                let line_note = to_line_note(*note, start);
                start = start.add_sixteenths(line_note.duration);
                line_notes.push(line_note);
            }
            Value::Rest => {
                start = start.add_sixteenths(1);
            }
        }
    }

    Line::new(line_notes)
}

fn to_line_note(note: Note, start: BeatNumber) -> LineNote {
    LineNote {
        start,
        duration: note.duration,
        note: note.to_wmidi_note(),
    }
}

#[cfg(test)]
mod tests {
    use wmidi::Note;

    use crate::{BeatNumber, Line, LineNote};

    use super::*;

    #[test]
    fn it_parses_line_starting_on_downbeat() {
        assert_eq!(
            parse_line("C4 F-1 G3 Bb3 C4 Db4 Eb4 F4 E4").unwrap(),
            Line::new(vec![
                LineNote {
                    start: BeatNumber { sixteenth_note: 0 },
                    duration: 1,
                    note: Note::C4,
                },
                LineNote {
                    start: BeatNumber { sixteenth_note: 1 },
                    duration: 1,
                    note: Note::FMinus1,
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
            parse_line("C4 F3 G3 Bb3 C4 Db4 Eb4 F4 E4 . . .").unwrap(),
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

    #[test]
    fn it_parses_trailing_rests() {
        assert_eq!(
            parse_line("C4 F3 G3 Bb3 C4 Db4 Eb4 F4 E4 . . . - -").unwrap(),
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

    #[test]
    fn it_parses_leading_rests() {
        assert_eq!(
            parse_line("- Db4 Bb3 Db4 C4 . Bb3 G3 F3 Bb3 F3 Gb3 G3 Gb3 F3 G3 E3 . . .").unwrap(),
            Line::new(vec![
                LineNote {
                    start: BeatNumber { sixteenth_note: 1 },
                    duration: 1,
                    note: Note::Db4,
                },
                LineNote {
                    start: BeatNumber { sixteenth_note: 2 },
                    duration: 1,
                    note: Note::Bb3,
                },
                LineNote {
                    start: BeatNumber { sixteenth_note: 3 },
                    duration: 1,
                    note: Note::Db4,
                },
                LineNote {
                    start: BeatNumber { sixteenth_note: 4 },
                    duration: 2,
                    note: Note::C4,
                },
                LineNote {
                    start: BeatNumber { sixteenth_note: 6 },
                    duration: 1,
                    note: Note::Bb3,
                },
                LineNote {
                    start: BeatNumber { sixteenth_note: 7 },
                    duration: 1,
                    note: Note::G3,
                },
                LineNote {
                    start: BeatNumber { sixteenth_note: 8 },
                    duration: 1,
                    note: Note::F3,
                },
                LineNote {
                    start: BeatNumber { sixteenth_note: 9 },
                    duration: 1,
                    note: Note::Bb3,
                },
                LineNote {
                    start: BeatNumber { sixteenth_note: 10 },
                    duration: 1,
                    note: Note::F3,
                },
                LineNote {
                    start: BeatNumber { sixteenth_note: 11 },
                    duration: 1,
                    note: Note::Gb3,
                },
                LineNote {
                    start: BeatNumber { sixteenth_note: 12 },
                    duration: 1,
                    note: Note::G3,
                },
                LineNote {
                    start: BeatNumber { sixteenth_note: 13 },
                    duration: 1,
                    note: Note::Gb3,
                },
                LineNote {
                    start: BeatNumber { sixteenth_note: 14 },
                    duration: 1,
                    note: Note::F3,
                },
                LineNote {
                    start: BeatNumber { sixteenth_note: 15 },
                    duration: 1,
                    note: Note::G3,
                },
                LineNote {
                    start: BeatNumber { sixteenth_note: 0 },
                    duration: 4,
                    note: Note::E3,
                },
            ])
        )
    }
}
