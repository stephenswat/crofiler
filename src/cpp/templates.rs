//! Things that could be templates

use crate::cpp::{
    types::{self, TypeLike},
    values::{self, ValueLike},
    IResult,
};
use nom::Parser;
use nom_supreme::ParserExt;

/// Parser recognizing a set of template parameters
///
/// None will be returned upon encountering the invalid "<, void>" pattern,
/// which clang unfortunately occasionally emits...
pub fn template_parameters(s: &str) -> IResult<Option<Box<[TemplateParameter]>>> {
    use nom::{
        character::complete::{char, space0},
        multi::separated_list0,
        sequence::delimited,
    };
    use nom_supreme::tag::complete::tag;
    let arguments = separated_list0(char(',').and(space0), template_parameter);
    (delimited(char('<'), arguments, space0.and(char('>'))).map(|v| Some(v.into_boxed_slice())))
        .or(tag("<, void>").value(None))
        .parse(s)
}

/// Parser recognizing a single template parameter/argument
fn template_parameter(s: &str) -> IResult<TemplateParameter> {
    let type_like = types::type_like.map(TemplateParameter::TypeLike);
    let value_like = values::value_like.map(TemplateParameter::ValueLike);
    type_like.or(value_like).parse(s)
}
//
/// Template parameter
#[derive(Debug, PartialEq, Clone)]
pub enum TemplateParameter<'source> {
    /// Type or value looking close enough to a type
    TypeLike(TypeLike<'source>),

    /// Value
    ValueLike(ValueLike),
}
//
impl From<ValueLike> for TemplateParameter<'_> {
    fn from(v: ValueLike) -> Self {
        Self::ValueLike(v)
    }
}
//
impl<'source> From<TypeLike<'source>> for TemplateParameter<'source> {
    fn from(t: TypeLike<'source>) -> Self {
        Self::TypeLike(t)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpp::tests::force_parse_type;
    use pretty_assertions::assert_eq;

    #[test]
    fn template_parameter() {
        fn test_template_parameter_sep(text_wo_sep: &str, sep: &str, expected: TemplateParameter) {
            let mut text = text_wo_sep.to_owned();
            text.push_str(sep);
            assert_eq!(super::template_parameter(&text), Ok((sep, expected)));
        }
        fn test_template_parameter(text_wo_sep: &str, expected: TemplateParameter) {
            test_template_parameter_sep(text_wo_sep, ",", expected.clone());
            test_template_parameter_sep(text_wo_sep, ">", expected);
        }
        test_template_parameter(
            &(i64::MIN.to_string()),
            TemplateParameter::ValueLike(i64::MIN.into()),
        );
        fn test_type_parameter(s: &str) {
            test_template_parameter(s, force_parse_type(s).into());
        }
        test_type_parameter("signed char*");
        test_type_parameter("charamel<lol>&");
    }

    #[test]
    fn template_parameters() {
        assert_eq!(
            super::template_parameters("<>"),
            Ok(("", Some(vec![].into())))
        );
        assert_eq!(
            super::template_parameters("<T>"),
            Ok(("", Some(vec![force_parse_type("T").into()].into())))
        );
        assert_eq!(
            super::template_parameters("<char, stuff>"),
            Ok((
                "",
                Some(
                    vec![
                        force_parse_type("char").into(),
                        force_parse_type("stuff").into()
                    ]
                    .into()
                )
            ))
        );
        assert_eq!(super::template_parameters("<, void>"), Ok(("", None)));
    }
}
