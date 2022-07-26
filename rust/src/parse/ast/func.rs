use std::collections::BTreeMap;
use crate::parse::ast::FnBodyType::{CODEBLOCK, COMPILER_PROVIDED};
use crate::parse::ast::{AstNode, CodeBlock, CrabType, Expression, Ident, Statement, StructId};
use crate::parse::{ParseError, Result, Rule};
use crate::try_from_pair;
use crate::util::{int_struct_name, main_func_name, ListFunctional, magic_main_func_name};
use pest::iterators::Pair;
use std::convert::TryFrom;
use crate::util::MapFunctional;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Func {
    pub signature: FuncSignature,
    pub body: FnBodyType,
}
try_from_pair!(Func, Rule::function);
impl AstNode for Func {
    fn from_pair(pair: Pair<Rule>) -> Result<Self> {
        let mut inner = pair.into_inner();
        let sig_pair = inner.next().ok_or(ParseError::ExpectedInner)?;
        let signature = FuncSignature::try_from(sig_pair)?;
        let body_pair = inner.next().ok_or(ParseError::ExpectedInner)?;
        let body = match body_pair.as_rule() {
            Rule::compiler_provided => Ok(COMPILER_PROVIDED),
            Rule::code_block => {
                let mut body = CodeBlock::try_from(body_pair)?;
                // Void functions should always have an implied return statement at the end
                if signature.return_type == CrabType::VOID {
                    body.statements.push(Statement::RETURN(None));
                }
                Ok(CODEBLOCK(body))
            }
            r => Err(ParseError::IncorrectRule(
                String::from("BodyType"),
                String::from("compiler_provided or codeblock"),
                format!("{:#?}", r),
            )),
        }?;

        Ok(Func { signature, body })
    }
}
impl Func {
    ///
    /// Convert this function to a method
    ///
    pub fn method(self, struct_id: StructId) -> Self {
        Self {
            body: self.body,
            signature: self.signature.method(struct_id),
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum FnBodyType {
    CODEBLOCK(CodeBlock),
    COMPILER_PROVIDED,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct FuncSignature {
    pub name: Ident,
    pub return_type: CrabType,
    pub pos_params: Vec<PosParam>,
    pub named_params: BTreeMap<Ident, NamedParam>,
    pub caller_id: Option<StructId>,
}

try_from_pair!(FuncSignature, Rule::fn_signature);
impl AstNode for FuncSignature {
    fn from_pair(pair: Pair<Rule>) -> Result<Self> {
        let mut inner = pair.into_inner();
        let name = Ident::from(inner.next().ok_or(ParseError::ExpectedInner)?.as_str());

        let (pos_params, named_params, return_type) = inner.try_fold(
            (vec![], BTreeMap::new(), CrabType::VOID),
            |(pos_params, named_params, return_type), pair| {
                Result::Ok(match pair.as_rule() {
                    Rule::pos_params => (PosParams::try_from(pair)?.0, named_params, return_type),
                    Rule::named_params => (pos_params, NamedParams::try_from(pair)?.0, return_type),
                    Rule::return_type => (pos_params, named_params, ReturnType::try_from(pair)?.0),
                    _ => {
                        return Err(ParseError::IncorrectRule(
                            String::from(stringify!(FuncSignature)),
                            format!(
                                "{:?} or {:?} or {:?}",
                                Rule::pos_params,
                                Rule::named_params,
                                Rule::return_type
                            ),
                            format!("{:?}", pair.as_rule()),
                        ))
                    }
                })
            },
        )?;

        let new_fn = Self {
            name,
            return_type,
            pos_params,
            named_params,
            caller_id: None,
        };

        let new_fn = if new_fn.verify_main_fn()? {
            Self {
                name: magic_main_func_name(),
                ..new_fn
            }
        } else {
            new_fn
        };

        Ok(new_fn)
    }
}
impl FuncSignature {
    ///
    /// Convert this function signature to a method
    ///
    pub(super) fn method(self, caller_id: StructId) -> Self {
        Self {
            caller_id: Some(caller_id),
            ..self
        }
    }

    fn verify_main_fn(&self) -> Result<bool> {
        if self.name == main_func_name() {
            if self.return_type != CrabType::SIMPLE(int_struct_name())
                || !self.pos_params.is_empty()
                || !self.named_params.is_empty()
            {
                Err(ParseError::MainSignature)
            } else {
                Ok(true)
            }
        } else {
            Ok(false)
        }
    }
}

struct ReturnType(CrabType);
try_from_pair!(ReturnType, Rule::return_type);
impl AstNode for ReturnType {
    fn from_pair(pair: Pair<Rule>) -> Result<Self>
    where
        Self: Sized,
    {
        Ok(Self(match pair.into_inner().next() {
            Some(ct) => CrabType::try_from(ct)?,
            None => CrabType::VOID,
        }))
    }
}

struct PosParams(Vec<PosParam>);
try_from_pair!(PosParams, Rule::pos_params);
impl AstNode for PosParams {
    fn from_pair(pair: Pair<Rule>) -> Result<Self>
    where
        Self: Sized,
    {
        Ok(Self(
            pair.into_inner().try_fold(vec![], |params, param| {
                Result::Ok(params.fpush(PosParam::try_from(param)?))
            })?,
        ))
    }
}

struct NamedParams(BTreeMap<Ident, NamedParam>);
try_from_pair!(NamedParams, Rule::named_params);
impl AstNode for NamedParams {
    fn from_pair(pair: Pair<Rule>) -> Result<Self>
    where
        Self: Sized,
    {
        Ok(Self(
            pair.into_inner().try_fold(BTreeMap::new(), |params, param| {
                let np = NamedParam::try_from(param)?;
                Result::Ok(params.finsert(np.name.clone(), np))
            })?,
        ))
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct PosParam {
    pub name: Ident,
    pub crab_type: CrabType,
}
try_from_pair!(PosParam, Rule::pos_param);
impl AstNode for PosParam {
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
impl From<NamedParam> for PosParam {
    fn from(nfp: NamedParam) -> Self {
        Self {
            name: nfp.name,
            crab_type: nfp.crab_type,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct NamedParam {
    pub name: Ident,
    pub crab_type: CrabType,
    pub expr: Expression,
}
try_from_pair!(NamedParam, Rule::named_param);
impl AstNode for NamedParam {
    fn from_pair(pair: Pair<Rule>) -> Result<Self>
    where
        Self: Sized,
    {
        let mut inner = pair.into_inner();
        let crab_type = CrabType::try_from(inner.next().ok_or(ParseError::ExpectedInner)?)?;
        let name = Ident::from(inner.next().ok_or(ParseError::ExpectedInner)?.as_str());
        let expr = Expression::try_from(inner.next().ok_or(ParseError::ExpectedInner)?)?;

        Ok(Self {
            name,
            crab_type,
            expr,
        })
    }
}
