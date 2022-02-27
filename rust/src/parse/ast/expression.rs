use crate::parse::ast::{
    AstNode, ExpressionChain, ExpressionChainType, FnCall, Primitive, StructFieldInit, StructInit,
};
use crate::parse::{ParseError, Result, Rule};
use crate::try_from_pair;
use crate::util::{int_struct_name, new_string_name, primitive_field_name};
use pest::iterators::Pair;
use std::convert::TryFrom;

#[derive(Debug, Clone, Eq, PartialEq)]
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
                match &prim {
                    Primitive::UINT(_) => Ok(Expression::STRUCT_INIT(StructInit {
                        name: int_struct_name(),
                        fields: vec![StructFieldInit {
                            name: primitive_field_name(),
                            value: Expression::PRIM(prim),
                        }],
                    })),
                    Primitive::STRING(str) => Ok(Expression::CHAIN(ExpressionChain {
                        this: ExpressionChainType::FN_CALL(FnCall {
                            name: new_string_name(),
                            unnamed_args: vec![
                                Expression::PRIM(prim.clone()),
                                Expression::PRIM(Primitive::UINT((str.len() + 1) as u64)),
                            ],
                            named_args: vec![],
                        }),
                        next: None,
                    })),
                    _ => Ok(Expression::PRIM(prim)),
                }
            }
            Rule::struct_init => Ok(Expression::STRUCT_INIT(StructInit::try_from(expr_type)?)),
            Rule::expression_chain => Ok(Expression::CHAIN(ExpressionChain::try_from(expr_type)?)),
            _ => Err(ParseError::NoMatch(String::from("Expression::from_pair"))),
        };
    }
}
