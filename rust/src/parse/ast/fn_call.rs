use crate::parse::ast::{AstNode, CrabType, Expression, Ident, StructId};
use crate::parse::{ParseError, Result, Rule};
use crate::util::ListFunctional;
use crate::{compile, try_from_pair};
use pest::iterators::Pair;
use std::convert::TryFrom;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct FnCall {
    pub name: Ident,
    pub tmpls: Vec<CrabType>,
    pub pos_args: Vec<Expression>,
    pub named_args: Vec<NamedArg>,
}
try_from_pair!(FnCall, Rule::fn_call);
impl AstNode for FnCall {
    fn from_pair(pair: Pair<Rule>) -> Result<Self>
    where
        Self: Sized,
    {
        let mut inner = pair.into_inner();
        let name = Ident::from(inner.next().ok_or(ParseError::ExpectedInner)?.as_str());
        let tmpls = Tmpls::try_from(inner.next().ok_or(ParseError::ExpectedInner)?)?.0;
        let (pos_args, named_args) =
            inner.try_fold((vec![], vec![]), |(pos_args, named_args), pair| {
                Ok(match pair.as_rule() {
                    Rule::pos_args => (PosArgs::try_from(pair)?.0, named_args),
                    Rule::named_args => (pos_args, NamedArgs::try_from(pair)?.0),
                    _ => {
                        return Err(ParseError::IncorrectRule(
                            String::from(stringify!(FnCall)),
                            format!("{:?} or {:?}", Rule::pos_args, Rule::named_args),
                            format!("{:?}", pair.as_rule()),
                        ))
                    }
                })
            })?;

        Ok(Self {
            name,
            tmpls,
            named_args,
            pos_args,
        })
    }
}
impl FnCall {
    pub(super) fn resolve(self, caller: CrabType, caller_id: &StructId) -> compile::Result<Self> {
        Ok(Self {
            pos_args: self
                .pos_args
                .into_iter()
                .try_fold(vec![], |pos_args, pos_arg| {
                    compile::Result::Ok(pos_args.fpush(pos_arg.resolve(caller.clone(), caller_id)?))
                })?,
            named_args: self
                .named_args
                .into_iter()
                .try_fold(vec![], |named_args, named_arg| {
                    compile::Result::Ok(
                        named_args.fpush(named_arg.resolve(caller.clone(), caller_id)?),
                    )
                })?,
            ..self
        })
    }
}

struct PosArgs(Vec<Expression>);
try_from_pair!(PosArgs, Rule::pos_args);
impl AstNode for PosArgs {
    fn from_pair(pair: Pair<Rule>) -> Result<Self>
    where
        Self: Sized,
    {
        Ok(Self(pair.into_inner().try_fold(vec![], |args, arg| {
            Result::Ok(args.fpush(PosArg::try_from(arg)?.0))
        })?))
    }
}

struct NamedArgs(Vec<NamedArg>);
try_from_pair!(NamedArgs, Rule::named_args);
impl AstNode for NamedArgs {
    fn from_pair(pair: Pair<Rule>) -> Result<Self>
    where
        Self: Sized,
    {
        Ok(Self(pair.into_inner().try_fold(vec![], |args, arg| {
            Result::Ok(args.fpush(NamedArg::try_from(arg)?))
        })?))
    }
}

struct PosArg(Expression);
try_from_pair!(PosArg, Rule::pos_arg);
impl AstNode for PosArg {
    fn from_pair(pair: Pair<Rule>) -> Result<Self>
    where
        Self: Sized,
    {
        Ok(Self(Expression::try_from(
            pair.into_inner().next().ok_or(ParseError::ExpectedInner)?,
        )?))
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct NamedArg {
    pub name: Ident,
    pub expr: Expression,
}
try_from_pair!(NamedArg, Rule::named_arg);
impl AstNode for NamedArg {
    fn from_pair(pair: Pair<Rule>) -> Result<Self>
    where
        Self: Sized,
    {
        let mut inner = pair.into_inner();
        let name = Ident::from(inner.next().ok_or(ParseError::ExpectedInner)?.as_str());
        let expr = Expression::try_from(inner.next().ok_or(ParseError::ExpectedInner)?)?;

        Ok(Self { name, expr })
    }
}
impl NamedArg {
    pub(super) fn resolve(self, caller: CrabType, caller_id: &StructId) -> compile::Result<Self> {
        Ok(Self {
            expr: self.expr.resolve(caller, caller_id)?,
            ..self
        })
    }
}

struct Tmpls(Vec<CrabType>);
try_from_pair!(Tmpls, Rule::tmpls);
impl AstNode for Tmpls {
    fn from_pair(pair: Pair<Rule>) -> Result<Self>
    where
        Self: Sized,
    {
        Ok(Self(
            pair.into_inner().try_fold(vec![], |tmpls, pair| {
                Result::Ok(tmpls.fpush(CrabType::try_from(pair)?))
            })?,
        ))
    }
}
