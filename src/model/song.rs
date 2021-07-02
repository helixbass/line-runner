use crate::Chord;
use combine::{parser::char::spaces, sep_by, Parser, Stream};
use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Song {
    pub chords: Vec<Chord>,
}

impl Song {
    pub fn new(chords: &[Chord]) -> Self {
        Self {
            chords: chords.to_vec(),
        }
    }

    pub fn parser<Input>() -> impl Parser<Input, Output = Self>
    where
        Input: Stream<Token = char>,
    {
        sep_by(Chord::parser(), spaces()).map(|chords: Vec<_>| Self::new(&chords))
    }
}

impl fmt::Display for Song {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let string = self
            .chords
            .iter()
            .map(|c| c.to_string())
            .collect::<Vec<_>>()
            .join(" ");
        f.write_str(&string)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parser() {
        let songs = vec!["A Bm CM7 D7 Em7".to_string()];

        let parsed: Vec<_> = songs
            .iter()
            .map(|s| Song::parser::<&str>().parse(s).unwrap().0)
            .map(|s| s.to_string())
            .collect();

        assert_eq!(parsed, songs)
    }
}
