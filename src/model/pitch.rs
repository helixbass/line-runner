use crate::{Letter, Modifier};
use combine::{Parser, Stream};
use std::fmt;
use strum::IntoEnumIterator;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Pitch {
    pub letter: Letter,
    pub modifier: Modifier,
}

impl Pitch {
    pub fn new(letter: Letter, modifier: Modifier) -> Self {
        Self { letter, modifier }
    }

    pub fn parser<Input>() -> impl Parser<Input, Output = Self>
    where
        Input: Stream<Token = char>,
    {
        (Letter::parser(), Modifier::parser()).map(|(letter, modifier)| Self::new(letter, modifier))
    }

    pub fn all() -> impl Iterator<Item = Pitch> {
        Letter::iter().flat_map(|l| Modifier::iter().map(move |m| Pitch::new(l, m)))
    }

    pub fn index(&self) -> i8 {
        match (self.letter, self.modifier) {
            (Letter::C, Modifier::Natural) => 0,
            (Letter::D, Modifier::Flat) => 1,
            (Letter::D, Modifier::Natural) => 2,
            (Letter::E, Modifier::Flat) => 3,
            (Letter::E, Modifier::Natural) => 4,
            (Letter::F, Modifier::Flat) => 4,
            (Letter::F, Modifier::Natural) => 5,
            (Letter::G, Modifier::Flat) => 6,
            (Letter::G, Modifier::Natural) => 7,
            (Letter::A, Modifier::Flat) => 8,
            (Letter::A, Modifier::Natural) => 9,
            (Letter::B, Modifier::Flat) => 10,
            (Letter::B, Modifier::Natural) => 11,
            (Letter::C, Modifier::Flat) => 11,
        }
    }
}

impl fmt::Display for Pitch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&format!("{}{}", self.letter, self.modifier))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parser() {
        let parsed: Vec<_> = Pitch::all()
            .map(|pitch| pitch.to_string())
            .map(|string| Pitch::parser::<&str>().parse(&string).unwrap().0)
            .collect();

        assert_eq!(parsed, Pitch::all().collect::<Vec<_>>());
    }

    #[test]
    fn index() {
        assert_eq!(Pitch::new(Letter::G, Modifier::Natural).index(), 7);
    }
}
