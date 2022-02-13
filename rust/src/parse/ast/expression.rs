use crate::parse::ast::{AstNode, FnCall, Ident, int_struct_name, Primitive, primitive_field_name, StructFieldInit, StructInit};
use crate::parse::{ParseError, Result, Rule};
use crate::try_from_pair;
use pest::iterators::Pair;
use std::convert::TryFrom;

#[derive(Debug, Clone)]
#[allow(non_camel_case_types)]
pub enum Expression {
    PRIM(Primitive),
    STRUCT_INIT(StructInit),
    CHAIN(ExpressionChain),
}

try_from_pair!(Expression, Rule::expression);
#[allow(unreachable_patterns)]
impl AstNode for Expression {
    fn from_pair(pair: Pair<Rule>) -> Result<Self>
    where
        Self: Sized,
    {
        let mut inner = pair.into_inner();
        let expr_type = inner.next().ok_or(ParseError::ExpectedInner)?;
        if !inner.next().is_none() {
            return Err(ParseError::UnexpectedInner);
        }

        return match expr_type.as_rule() {
            Rule::primitive => {
                let prim = Primitive::try_from(expr_type)?;
                match prim {
                    Primitive::UINT(_) => Ok(Expression::STRUCT_INIT(StructInit {
                        name: int_struct_name(),
                        fields: vec![StructFieldInit {
                            name: primitive_field_name(),
                            value: Expression::PRIM(prim)
                        }]
                    })),
                    _ => Ok(Expression::PRIM(prim))
                }
            },
            Rule::struct_init => Ok(Expression::STRUCT_INIT(StructInit::try_from(expr_type)?)),
            Rule::expression_chain => Ok(Expression::CHAIN(ExpressionChain::try_from(expr_type)?)),
            _ => Err(ParseError::NoMatch(String::from("Expression::from_pair"))),
        };
    }
}

#[derive(Debug, Clone)]
pub struct ExpressionChain {
    pub this: ExpressionChainType,
    pub next: Option<Box<ExpressionChain>>,
}

try_from_pair!(ExpressionChain, Rule::expression_chain);
impl AstNode for ExpressionChain {
    fn from_pair(pair: Pair<Rule>) -> Result<Self>
    where
        Self: Sized,
    {
        let mut inner = pair.into_inner();
        let this = ExpressionChainType::try_from(inner.next().ok_or(ParseError::ExpectedInner)?)?;
        let mut expr_chain = Self {
            this: this,
            next: None,
        };

        for in_pair in inner {
            let next = ExpressionChainType::try_from(in_pair)?;
            expr_chain = expr_chain.append(next);
        }

        Ok(expr_chain)
    }
}
impl ExpressionChain {
    fn append(self, addition: ExpressionChainType) -> Self {
        match self.next {
            None => Self {
                this: self.this,
                next: Some(Box::new(Self {
                    this: addition,
                    next: None,
                })),
            },
            Some(ec) => Self {
                this: self.this,
                next: Some(Box::new(ec.append(addition))),
            },
        }
    }
}

#[derive(Debug, Clone)]
#[allow(non_camel_case_types)]
pub enum ExpressionChainType {
    FN_CALL(FnCall),
    VARIABLE(Ident),
}

/// ExpressionChainType requires a custom TryFrom implementation because it can be built from two different rules
impl TryFrom<Pair<'_, Rule>> for ExpressionChainType {
    type Error = ParseError;
    fn try_from(pair: Pair<Rule>) -> std::result::Result<ExpressionChainType, Self::Error> {
        match pair.as_rule() {
            Rule::ident => ExpressionChainType::from_pair(pair),
            Rule::fn_call => ExpressionChainType::from_pair(pair),
            _ => Err(ParseError::IncorrectRule(
                String::from(stringify!(ExpressionChainType)),
                format!(
                    "{} or {}",
                    stringify!(Rule::ident),
                    stringify!(Rule::fn_call)
                ),
                format!("{:?}", pair.as_rule()),
            )),
        }
    }
}
impl AstNode for ExpressionChainType {
    fn from_pair(pair: Pair<Rule>) -> Result<Self>
    where
        Self: Sized,
    {
        match pair.clone().as_rule() {
            Rule::ident => Ok(Self::VARIABLE(Ident::from(pair.as_str()))),
            Rule::fn_call => Ok(Self::FN_CALL(FnCall::try_from(pair)?)),
            _ => Err(ParseError::IncorrectRule(
                String::from(stringify!(ExpressionChainType)),
                format!(
                    "{:#?} or {:#?}",
                    stringify!(Rule::ident),
                    stringify!(Rule::fn_call)
                ),
                format!("{:#?}", pair.as_rule()),
            )),
        }
    }
}
