use crate::parse::ast::{AstNode, Ident};
use crate::parse::{ParseError, Rule};
use crate::{parse, try_from_pair};
use pest::iterators::Pair;
use std::convert::TryFrom;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Operator {
    ADD,
    SUB,
    MULT,
    DIV,
    EQ,
    LT,
    GT,
    LTE,
    GTE,
    LSH,
    RSH,
}

try_from_pair!(Operator, Rule::operator);
impl AstNode for Operator {
    fn from_pair(pair: Pair<Rule>) -> parse::Result<Self>
    where
        Self: Sized,
    {
        match pair.as_str() {
            "+" => Ok(Self::ADD),
            "-" => Ok(Self::SUB),
            "*" => Ok(Self::MULT),
            "/" => Ok(Self::DIV),
            "==" => Ok(Self::EQ),
            "<" => Ok(Self::LT),
            ">" => Ok(Self::GT),
            "<=" => Ok(Self::LTE),
            ">=" => Ok(Self::GTE),
            "<<" => Ok(Self::LSH),
            ">>" => Ok(Self::RSH),
            _ => unimplemented!(),
        }
    }
}
impl Operator {
    pub fn into_fn_name(self) -> Ident {
        Ident::from(match self {
            Self::ADD => "operatorAdd",
            Self::SUB => "operatorSub",
            Self::MULT => "operatorMult",
            Self::DIV => "operatorDiv",
            Self::EQ => "operatorEq",
            Self::LT => "operatorLt",
            Self::GT => "operatorGt",
            Self::LTE => "operatorLte",
            Self::GTE => "operatorGte",
            Self::LSH => "operatorLsh",
            Self::RSH => "operatorRsh",
        })
    }
}
