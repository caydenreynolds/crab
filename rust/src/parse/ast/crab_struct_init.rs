use crate::parse::ast::{AstNode, Expression, Ident};
use crate::parse::{ParseError, Result, Rule};
use crate::try_from_pair;
use pest::iterators::Pair;
use std::convert::TryFrom;
use crate::util::ListFunctional;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct StructInit {
    pub name: Ident,
    pub fields: Vec<StructFieldInit>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct StructFieldInit {
    pub name: Ident,
    pub value: Expression,
}

try_from_pair!(StructInit, Rule::struct_init);
impl AstNode for StructInit {
    fn from_pair(pair: Pair<Rule>) -> Result<Self>
    where
        Self: Sized,
    {
        let mut inner = pair.into_inner();
        let name = Ident::from(
            inner
                .next()
                .ok_or(ParseError::NoMatch(String::from("Struct::from_pair")))?
                .as_str(),
        );
        let fields = inner.try_fold(vec![], |fields, field| {
            Ok(fields.fpush(StructFieldInit::try_from(field)?))
        })?;
        Ok(Self { name, fields })
    }
}

try_from_pair!(StructFieldInit, Rule::struct_field_init);
impl AstNode for StructFieldInit {
    fn from_pair(pair: Pair<Rule>) -> Result<Self>
        where
            Self: Sized,
    {
        let mut inner = pair.into_inner();
        let name = Ident::from(inner.next().ok_or(ParseError::ExpectedInner)?.as_str());
        let value = Expression::try_from(inner.next().ok_or(ParseError::ExpectedInner)?)?;

        Ok(Self { name, value })
    }
}
