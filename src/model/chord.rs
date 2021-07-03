use crate::{Pitch, Quality};
use combine::{Parser, Stream};
use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Chord {
    pub pitch: Pitch,
    pub quality: Quality,
}

impl Chord {
    pub fn new(pitch: Pitch, quality: Quality) -> Self {
        Self { pitch, quality }
    }

    pub fn parser<Input>() -> impl Parser<Input, Output = Self>
    where
        Input: Stream<Token = char>,
    {
        (Pitch::parser(), Quality::parser()).map(|(pitch, quality)| Chord::new(pitch, quality))
    }
}

impl fmt::Display for Chord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&format!("{}{}", self.pitch, self.quality))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use strum::IntoEnumIterator;

    #[test]
    fn parser() {
        let chords: Vec<_> = Pitch::all()
            .into_iter()
            .flat_map(|pitch| Quality::iter().map(move |qquality| Chord::new(pitch, qquality)))
            .collect();

        let parsed: Vec<_> = chords
            .iter()
            .map(|chord| chord.to_string())
            .map(|string| Chord::parser::<&str>().parse(&string).unwrap().0)
            .collect();

        assert_eq!(parsed, chords);
    }
}
