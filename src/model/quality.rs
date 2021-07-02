use combine::{attempt, choice, optional, parser::char::string, Parser, Stream};
use strum_macros::{Display, EnumIter};
use Quality::*;

#[derive(Clone, Copy, Debug, Display, EnumIter, Eq, PartialEq)]
pub enum Quality {
    #[strum(serialize = "")]
    Major,
    #[strum(serialize = "m")]
    Minor,
    #[strum(serialize = "M7")]
    MajorSeventh,
    #[strum(serialize = "7")]
    Seventh,
    #[strum(serialize = "m7")]
    MinorSeventh,
}

impl Quality {
    pub fn parser<Input>() -> impl Parser<Input, Output = Self>
    where
        Input: Stream<Token = char>,
    {
        optional(choice((
            string("M7").map(|_| MajorSeventh),
            string("7").map(|_| Seventh),
            attempt(string("m7").map(|_| MinorSeventh)),
            string("m").map(|_| Minor),
        )))
        .map(|q| q.unwrap_or(Major))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use strum::IntoEnumIterator;

    #[test]
    fn parser() {
        let parsed: Vec<_> = Quality::iter()
            .map(|q| q.to_string())
            .map(|string| Quality::parser::<&str>().parse(&string).unwrap().0)
            .collect();
        let qualities: Vec<_> = Quality::iter().collect();

        assert_eq!(parsed, qualities);
    }
}
