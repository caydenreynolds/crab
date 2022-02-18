use crate::parse::ast::{AstNode, CodeBlock, CrabType, FuncSignature, Ident, Statement};
use crate::parse::{ParseError, Result, Rule};
use crate::try_from_pair;
use pest::iterators::Pair;
use std::convert::TryFrom;

#[derive(Debug, Clone)]
pub struct Func {
    pub signature: FuncSignature,
    pub body: CodeBlock,
}

try_from_pair!(Func, Rule::function);
impl AstNode for Func {
    fn from_pair(pair: Pair<Rule>) -> Result<Self> {
        let mut inner = pair.into_inner();
        let sig_pair = inner.next().ok_or(ParseError::ExpectedInner)?;
        let signature = FuncSignature::try_from(sig_pair)?;
        let body_pair = inner.next().ok_or(ParseError::ExpectedInner)?;
        let mut body = CodeBlock::try_from(body_pair)?;

        // Void functions should always have an implied return statement at the end
        if signature.return_type == CrabType::VOID {
            body.statements.push(Statement::RETURN(None));
        }

        Ok(Func { signature, body })
    }
}
impl Func {
    ///
    /// Convert this function to a method
    /// This works by adding an parameter of type struct_name to the beginning of this func's arguments
    ///
    pub fn method(self, struct_name: Ident) -> Self {
        Self {
            body: self.body,
            signature: self.signature.method(struct_name),
        }
    }

    pub fn with_mangled_name(self) -> Self {
        Self {
            body: self.body,
            signature: self.signature.with_mangled_name(),
        }
    }
}
