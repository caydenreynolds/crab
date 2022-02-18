use crate::parse::ast::{AstNode, Func, Struct, StructImpl};
use crate::parse::{ParseError, Rule};
use crate::{parse, try_from_pair};
use pest::iterators::Pair;
use std::convert::TryFrom;

#[derive(Debug, Clone)]
pub struct CrabAst {
    pub functions: Vec<Func>,
    pub structs: Vec<Struct>,
}

try_from_pair!(CrabAst, Rule::program);
impl AstNode for CrabAst {
    fn from_pair(pair: Pair<Rule>) -> parse::Result<Self> {
        let inner = pair.into_inner();
        let mut functions = vec![];
        let mut structs = vec![];
        let mut impls = vec![];

        for in_pair in inner {
            match in_pair.clone().as_rule() {
                Rule::function => functions.push(Func::try_from(in_pair)?.with_mangled_name()),
                Rule::crab_struct => structs.push(Struct::try_from(in_pair)?),
                Rule::impl_block => impls.push(StructImpl::try_from(in_pair)?),
                Rule::EOI => break, // If something shows up after EOI, we have a big problem
                _ => return Err(ParseError::NoMatch(String::from("CrabAst::from_pair"))),
            }
        }

        for struct_impl in impls {
            for func in struct_impl.fns {
                functions.push(func.method(struct_impl.struct_name.clone()));
            }
        }

        Ok(Self { functions, structs })
    }
}
