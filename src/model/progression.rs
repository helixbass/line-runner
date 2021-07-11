use crate::{Chord, Result};
use combine::{parser::char::spaces, sep_by, Parser, Stream};
use serde::{de, Deserialize, Deserializer};
use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Progression {
    pub chords: Vec<Chord>,
}

impl Progression {
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

    pub fn parse(string: &str) -> Result<Self> {
        let (result, _) = Self::parser::<&str>().parse(string)?;

        Ok(result)
    }
}

impl Default for Progression {
    fn default() -> Self {
        Self::parse("C").unwrap()
    }
}

impl fmt::Display for Progression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let string = self
            .chords
            .iter()
            .map(|chord| chord.to_string())
            .collect::<Vec<_>>()
            .join(" ");
        f.write_str(&string)
    }
}

impl<'de> Deserialize<'de> for Progression {
    fn deserialize<TDeserializer>(
        deserializer: TDeserializer,
    ) -> std::result::Result<Self, TDeserializer::Error>
    where
        TDeserializer: Deserializer<'de>,
    {
        let progression_string: String = Deserialize::deserialize(deserializer)?;
        Progression::parse(&progression_string).map_err(de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse() {
        let progressions = vec!["A Bm CM7 D7 Em7".to_string()];

        let parsed: Vec<_> = progressions
            .iter()
            .map(|string| Progression::parse(string).unwrap())
            .map(|s| s.to_string())
            .collect();

        assert_eq!(parsed, progressions)
    }
}
