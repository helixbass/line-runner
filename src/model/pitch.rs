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
}
