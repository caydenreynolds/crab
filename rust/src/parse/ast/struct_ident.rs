use crate::parse::ast::{AstNode, Expression, Ident};
use crate::parse::{ParseError, Result, Rule};
use crate::try_from_pair;
use pest::iterators::Pair;
use std::convert::TryFrom;
use std::fmt::{Display, Formatter};
use crate::util::ListFunctional;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct StructIdent {
    pub name: Ident,
    pub tmpls: Vec<Ident>,
}

try_from_pair!(StructIdent, Rule::struct_ident);
impl AstNode for StructIdent {
    fn from_pair(pair: Pair<Rule>) -> Result<Self>
        where
            Self: Sized,
    {
        let mut inner = pair.into_inner();
        let name = Ident::from(inner.next().ok_or(ParseError::ExpectedInner)?.as_str());
        let tmpls = inner.map(|pair| Ident::from(pair.as_str())).collect();
        Ok(Self { name, tmpls })
    }
}
impl Display for StructIdent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)?;

        if self.tmpls.len() > 0 {
            write!(f, "<")?;
            let (last, tmpls) = self.tmpls.split_last().unwrap();
            tmpls.iter().try_for_each(|tmpl| write!(f, "{}, ", tmpl))?;
            write!(f, "{}>", last)?;
        }

        Ok(())
    }
}
