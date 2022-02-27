use crate::parse::ast::{AstNode, CrabType, Ident, NamedFnParam};
use crate::parse::{ParseError, Result, Rule};
use crate::try_from_pair;
use pest::iterators::Pair;
use std::convert::TryFrom;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct FnParam {
    pub name: Ident,
    pub crab_type: CrabType,
}

try_from_pair!(FnParam, Rule::fn_param);
impl AstNode for FnParam {
    fn from_pair(pair: Pair<Rule>) -> Result<Self>
    where
        Self: Sized,
    {
        let mut inner = pair.into_inner();
        let crab_type = CrabType::try_from(inner.next().ok_or(ParseError::ExpectedInner)?)?;
        let name = Ident::from(inner.next().ok_or(ParseError::ExpectedInner)?.as_str());

        Ok(Self { name, crab_type })
    }
}

impl FnParam {
    ///
    /// Used to resolve FnParams with an interface name to FnParams with a StructName
    ///
    /// Params:
    /// * `ty` - The CrabType for the new FnParam
    ///
    /// Returns:
    /// An fn param with this FnParams's name and the given type
    ///
    pub fn with_type(self, ty: CrabType) -> Self {
        Self {
            crab_type: ty,
            ..self
        }
    }
}

impl From<NamedFnParam> for FnParam {
    fn from(nfp: NamedFnParam) -> Self {
        Self {
            name: nfp.name,
            crab_type: nfp.crab_type,
        }
    }
}
