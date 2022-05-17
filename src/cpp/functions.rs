//! Function-related parsing

use crate::cpp::{
    atoms,
    types::{
        self,
        qualifiers::{self, ConstVolatile, Reference},
        TypeLike,
    },
    values::{self, ValueLike},
    IResult,
};
use nom::Parser;

/// Parser recognizing a function signature (parameters + qualifiers)
pub fn function_signature(s: &str) -> IResult<FunctionSignature> {
    use nom::{
        character::complete::space0,
        combinator::opt,
        sequence::{preceded, tuple},
    };
    let cv = preceded(space0, qualifiers::cv);
    let reference = preceded(space0, qualifiers::reference);
    let noexcept = preceded(space0, opt(noexcept));
    tuple((function_parameters, cv, reference, noexcept))
        .map(|(parameters, cv, reference, noexcept)| FunctionSignature {
            parameters,
            cv,
            reference,
            noexcept,
        })
        .parse(s)
}
//
/// Function signature
#[derive(Clone, Default, Debug, PartialEq)]
pub struct FunctionSignature<'source> {
    /// Parameter types
    parameters: Box<[TypeLike<'source>]>,

    /// CV qualifiers
    cv: ConstVolatile,

    /// Reference qualifiers
    reference: Reference,

    /// noexcept qualifier
    ///
    /// The first layer of Option represents presence or absence of the
    /// "noexcept" keyword, the second layer represents the optional expression
    /// that can be passed as an argument to noexcept.
    noexcept: Option<Option<ValueLike<'source>>>,
}

/// Parser recognizing a set of function parameters
fn function_parameters(s: &str) -> IResult<Box<[TypeLike]>> {
    use nom::{
        character::complete::{char, space0},
        multi::separated_list0,
        sequence::delimited,
    };
    let parameters = separated_list0(space0.and(char(',')).and(space0), types::type_like);
    delimited(char('('), parameters, space0.and(char(')')))
        .map(Vec::into_boxed_slice)
        .parse(s)
}

/// Parser recognizing the noexcept qualifier and its optional argument
fn noexcept(s: &str) -> IResult<Option<ValueLike>> {
    use nom::{
        character::complete::char,
        combinator::opt,
        sequence::{delimited, preceded},
    };
    preceded(
        atoms::keyword("noexcept"),
        opt(delimited(char('('), values::value_like, char(')'))),
    )
    .parse(s)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpp::tests::force_parse_type;
    use pretty_assertions::assert_eq;

    #[test]
    fn noexcept() {
        assert_eq!(super::noexcept("noexcept"), Ok(("", None)));
        assert_eq!(
            super::noexcept("noexcept(123)"),
            Ok(("", Some(values::value_like("123").unwrap().1)))
        );
    }

    #[test]
    fn function_signature() {
        assert_eq!(
            super::function_signature("()"),
            Ok(("", FunctionSignature::default()))
        );
        assert_eq!(
            super::function_signature("() const"),
            Ok((
                "",
                FunctionSignature {
                    cv: ConstVolatile::CONST,
                    ..Default::default()
                }
            ))
        );
        assert_eq!(
            super::function_signature("() &&"),
            Ok((
                "",
                FunctionSignature {
                    reference: Reference::RValue,
                    ..Default::default()
                }
            ))
        );
        assert_eq!(
            super::function_signature("() noexcept"),
            Ok((
                "",
                FunctionSignature {
                    noexcept: Some(None),
                    ..Default::default()
                }
            ))
        );
        assert_eq!(
            super::function_signature("() volatile &"),
            Ok((
                "",
                FunctionSignature {
                    cv: ConstVolatile::VOLATILE,
                    reference: Reference::LValue,
                    ..Default::default()
                }
            ))
        );
        assert_eq!(
            super::function_signature("() volatile const noexcept"),
            Ok((
                "",
                FunctionSignature {
                    cv: ConstVolatile::CONST | ConstVolatile::VOLATILE,
                    noexcept: Some(None),
                    ..Default::default()
                }
            ))
        );
        assert_eq!(
            super::function_signature("() && noexcept(456)"),
            Ok((
                "",
                FunctionSignature {
                    reference: Reference::RValue,
                    noexcept: Some(Some(values::value_like("456").unwrap().1)),
                    ..Default::default()
                }
            ))
        );
    }

    #[test]
    fn function_parameters() {
        assert_eq!(super::function_parameters("()"), Ok(("", vec![].into())));
        assert_eq!(
            super::function_parameters("(signed char*)"),
            Ok(("", vec![force_parse_type("signed char*")].into()))
        );
        assert_eq!(
            super::function_parameters("(charamel<lol>&, T)"),
            Ok((
                "",
                vec![force_parse_type("charamel<lol>&"), force_parse_type("T")].into()
            ))
        );
    }
}
