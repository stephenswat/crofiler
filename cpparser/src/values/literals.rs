//! Literals (and things that should be literals like negative numbers)

use crate::{names::atoms, EntityParser, IResult};
use nom::Parser;
use nom_supreme::ParserExt;

impl EntityParser {
    /// Parser for literals
    pub fn parse_literal<'source>(
        &self,
        s: &'source str,
    ) -> IResult<'source, Literal<atoms::IdentifierKey>> {
        literal(s, |s| self.parse_identifier(s))
    }
}

/// Parser for literals
//
// TODO: Make private once users are migrated
pub fn literal<'source, IdentifierKey: 'source>(
    s: &'source str,
    parse_identifier: impl Fn(&'source str) -> IResult<IdentifierKey>,
) -> IResult<Literal<IdentifierKey>> {
    use nom::combinator::opt;
    (literal_value.and(opt(parse_identifier)))
        .map(|(value, custom_suffix)| Literal {
            value,
            custom_suffix,
        })
        .parse(s)
}

/// A modern C++ literal, accounting for custom literals
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Literal<IdentifierKey> {
    /// Inner value
    value: LiteralValue,

    /// Custom literal suffix, if any
    custom_suffix: Option<IdentifierKey>,
}
//
impl<T: Into<LiteralValue>, U> From<T> for Literal<U> {
    fn from(value: T) -> Self {
        Self {
            value: value.into(),
            custom_suffix: None,
        }
    }
}

/// Parser for literal values
fn literal_value(s: &str) -> IResult<LiteralValue> {
    let character = character.map(LiteralValue::Char);
    integer.or(character).parse(s)
}

/// A literal value, or something that looks close enough to it
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LiteralValue {
    /// Signed 64-bit integer
    I64(i64),

    /// Unsigned 64-bit integer
    U64(u64),

    /// Character
    Char(char),
}
//
// Can't just impl<I: Into<i64>> at it would break other From impls...
macro_rules! literal_i64_from_integer {
    ($($integer:ident),*) => {
        $(
            impl From<$integer> for LiteralValue {
                fn from(i: $integer) -> Self {
                    LiteralValue::I64(i.into())
                }
            }
        )*
    }
}
literal_i64_from_integer!(i8, u8, i16, u16, i32, u32, i64);
//
impl From<u64> for LiteralValue {
    fn from(i: u64) -> Self {
        LiteralValue::U64(i)
    }
}
//
impl From<char> for LiteralValue {
    fn from(c: char) -> Self {
        LiteralValue::Char(c)
    }
}

/// Parser recognizing C-style integer literals + negative numbers
fn integer(s: &str) -> IResult<LiteralValue> {
    use nom::{
        character::complete::{i64, satisfy, u64},
        multi::many0_count,
    };
    i64.map(LiteralValue::I64)
        .or(u64.map(LiteralValue::U64))
        .terminated(many0_count(satisfy(|c| {
            let c = c.to_ascii_uppercase();
            c == 'U' || c == 'L' || c == 'Z'
        })))
        .parse(s)
}

/// Parser recognizing C-style character literals
fn character(s: &str) -> IResult<char> {
    use nom::{
        character::complete::{anychar, char},
        combinator::opt,
        sequence::{delimited, preceded},
    };
    use nom_supreme::tag::complete::tag;
    let prefix = opt(tag("u8")
        .value('8')
        .or(char('u'))
        .or(char('U'))
        .or(char('L')));
    let escape_sequence = (char('t').value('\t'))
        .or(char('r').value('\r'))
        .or(char('n').value('\n'))
        .or(char('\''))
        .or(char('"'))
        .or(char('\\'));
    delimited(
        prefix.and(char('\'')),
        preceded(char('\\'), escape_sequence).or(anychar),
        char('\''),
    )(s)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use std::fmt::Write;

    #[test]
    fn integer() {
        fn test_integer(num: impl Into<i128>) {
            let num: i128 = num.into();
            let num_str = num.to_string();
            for unsigned_suffix in ["", "U"] {
                for size_suffix in ["", "L", "LL", "Z"] {
                    for lowercase in [false, true] {
                        for size_first in [false, true] {
                            let suffix = if size_first {
                                size_suffix.chars().chain(unsigned_suffix.chars())
                            } else {
                                unsigned_suffix.chars().chain(size_suffix.chars())
                            };
                            let case = |c: char| {
                                if lowercase {
                                    c.to_ascii_lowercase()
                                } else {
                                    c
                                }
                            };
                            let mut num_str = num_str.clone();
                            for c in suffix.map(case) {
                                num_str.push(c);
                            }
                            let result: IResult<i128> = match super::integer(&num_str) {
                                Ok((x, LiteralValue::I64(i))) => Ok((x, i.into())),
                                Ok((x, LiteralValue::U64(u))) => Ok((x, u.into())),
                                Ok((_, LiteralValue::Char(c))) => {
                                    panic!("Unexpected char {c} in integer parser output")
                                }
                                Err(e) => Err(e),
                            };
                            assert_eq!(result, Ok(("", num)));
                        }
                    }
                }
            }
        }
        test_integer(i64::MIN);
        test_integer(u64::MAX);
    }

    #[test]
    fn character() {
        fn test_character_str(c: char) {
            for prefix in ["", "u8", "u", "U", "L"] {
                let mut char_str = prefix.to_string();
                write!(&mut char_str, "'{}'", c.escape_default()).unwrap();
                println!("{char_str}");
                assert_eq!(super::character(&char_str), Ok(("", c)));
            }
        }
        test_character_str('x');
        test_character_str('\t');
        test_character_str('\n');
        test_character_str('\'');
    }

    #[test]
    fn literal_value() {
        assert_eq!(super::literal_value("-123"), Ok(("", (-123i8).into())));
        assert_eq!(super::literal_value("'c'"), Ok(("", 'c'.into())));
    }

    #[test]
    fn literal() {
        let parse_literal = |s| super::literal(s, atoms::identifier);
        assert_eq!(parse_literal("'x'"), Ok(("", 'x'.into())));
        assert_eq!(
            parse_literal("42_m"),
            Ok((
                "",
                Literal {
                    value: 42u8.into(),
                    custom_suffix: Some("_m")
                }
            ))
        );
    }
}
