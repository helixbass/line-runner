use combine::{choice, token, Parser, Stream};
use strum_macros::{Display, EnumIter};

#[derive(Clone, Copy, Debug, Display, EnumIter, Eq, PartialEq)]
pub enum Letter {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
}

impl Letter {
    pub fn parser<Input>() -> impl Parser<Input, Output = Self>
    where
        Input: Stream<Token = char>,
    {
        choice((
            token('A').map(|_| Letter::A),
            token('B').map(|_| Letter::B),
            token('C').map(|_| Letter::C),
            token('D').map(|_| Letter::D),
            token('E').map(|_| Letter::E),
            token('F').map(|_| Letter::F),
            token('G').map(|_| Letter::G),
        ))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use strum::IntoEnumIterator;

    #[test]
    fn parser() {
        let parsed: Vec<_> = Letter::iter()
            .map(|l| l.to_string())
            .map(|string| Letter::parser::<&str>().parse(&string).unwrap().0)
            .collect();
        let letters: Vec<_> = Letter::iter().collect();

        assert_eq!(parsed, letters);
    }
}
