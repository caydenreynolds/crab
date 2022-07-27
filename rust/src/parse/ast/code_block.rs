use crate::parse::ast::{AstNode, CrabType, Statement};
use crate::parse::{ParseError, Result, Rule};
use crate::{compile, try_from_pair};
use pest::iterators::Pair;
use std::convert::TryFrom;
use crate::util::ListFunctional;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CodeBlock {
    pub statements: Vec<Statement>,
}

try_from_pair!(CodeBlock, Rule::code_block);
impl AstNode for CodeBlock {
    fn from_pair(pair: Pair<Rule>) -> Result<Self> {
        let mut statements = vec![];
        for statement in pair.into_inner() {
            match Statement::try_from(statement) {
                Ok(stmt) => statements.push(stmt),
                Err(err) => {
                    match err {
                        ParseError::None => {} // Do nothing
                        _ => return Err(err),
                    }
                }
            }
        }
        Ok(CodeBlock { statements })
    }
}
impl CodeBlock {
    pub(super) fn resolve(self, caller: CrabType) -> compile::Result<Self> {
        Ok(Self {
            statements: self
                .statements
                .into_iter()
                .try_fold(vec![], |statements, statement| {
                    compile::Result::Ok(statements.fpush(statement.resolve(caller)?))
                })?
        })
    }
}