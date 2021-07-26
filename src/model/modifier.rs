use combine::{optional, token, Parser, Stream};
use core::fmt;
use strum_macros::EnumIter;
use Modifier::*;

#[derive(Clone, Copy, Debug, EnumIter, Eq, PartialEq)]
pub enum Modifier {
    Flat,
    Natural,
}

impl Modifier {
    pub fn parser<Input>() -> impl Parser<Input, Output = Self>
    where
        Input: Stream<Token = char>,
    {
        let flat_parser = token('b').map(|_| Flat);

        optional(flat_parser).map(|modifier| modifier.unwrap_or(Natural))
    }
}

impl fmt::Display for Modifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let string = match self {
            Flat => "b",
            Natural => "",
        };

        f.write_str(string)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use strum::IntoEnumIterator;

    #[test]
    fn parser() {
        let parsed: Vec<_> = Modifier::iter()
            .map(|m| m.to_string())
            .map(|string| Modifier::parser::<&str>().parse(&string).unwrap().0)
            .collect();
        let letters: Vec<_> = Modifier::iter().collect();

        assert_eq!(parsed, letters);
    }
}
