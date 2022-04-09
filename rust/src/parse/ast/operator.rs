use crate::parse::ast::{AstNode, Ident};
use crate::parse::{ParseError, Rule};
use crate::util::{
    operator_add_name, operator_div_name, operator_eq_name, operator_gt_name, operator_gte_name,
    operator_lsh_name, operator_lt_name, operator_lte_name, operator_mult_name, operator_rsh_name,
    operator_sub_name,
};
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
        match self {
            Self::ADD => operator_add_name(),
            Self::SUB => operator_sub_name(),
            Self::MULT => operator_mult_name(),
            Self::DIV => operator_div_name(),
            Self::EQ => operator_eq_name(),
            Self::LT => operator_lt_name(),
            Self::GT => operator_gt_name(),
            Self::LTE => operator_lte_name(),
            Self::GTE => operator_gte_name(),
            Self::LSH => operator_lsh_name(),
            Self::RSH => operator_rsh_name(),
        }
    }
}
