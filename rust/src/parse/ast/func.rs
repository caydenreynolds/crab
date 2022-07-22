use crate::parse::ast::FnBodyType::{CODEBLOCK, COMPILER_PROVIDED};
use crate::parse::ast::{AstNode, CodeBlock, CrabType, Expression, Ident, Statement};
use crate::parse::{ParseError, Result, Rule};
use crate::try_from_pair;
use pest::iterators::Pair;
use std::convert::TryFrom;
use crate::util::{int_struct_name, main_func_name, mangle_function_name};

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

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum FnBodyType {
    CODEBLOCK(CodeBlock),
    COMPILER_PROVIDED,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct FuncSignature {
    pub name: Ident,
    pub return_type: CrabType,
    pub pos_params: Vec<PosParam>,
    pub named_params: Vec<NamedParam>,
}

try_from_pair!(FuncSignature, Rule::fn_signature);
impl AstNode for FuncSignature {
    fn from_pair(pair: Pair<Rule>) -> Result<Self> {
        let mut inner = pair.into_inner();
        let name = Ident::from(inner.next().ok_or(ParseError::ExpectedInner)?.as_str());
        let mut return_type_option = None;
        let mut unnamed_params = vec![];
        let mut named_params = vec![];
        let mut seen_named_param = false;

        for inner_pair in inner {
            match inner_pair.clone().as_rule() {
                Rule::crab_type => return_type_option = Some(CrabType::try_from(inner_pair)?),
                Rule::fn_param => {
                    unnamed_params.push(PosParam::try_from(inner_pair)?);
                    if seen_named_param {
                        return Err(ParseError::PositionalParamAfterNamedParam(
                            name.clone(),
                            unnamed_params
                                .get(unnamed_params.len() - 1)
                                .unwrap()
                                .name
                                .clone(),
                        ));
                    }
                }
                Rule::named_fn_param => {
                    named_params.push(NamedParam::try_from(inner_pair)?);
                    seen_named_param = true;
                }
                _ => {
                    return Err(ParseError::NoMatch(String::from(
                        "FuncSignature::from_pair",
                    )))
                }
            }
        }
        let unnamed_params = unnamed_params;
        let named_params = named_params;

        let return_type = match return_type_option {
            None => CrabType::VOID,
            Some(ct) => ct,
        };

        let new_fn = Self {
            name,
            return_type,
            pos_params: unnamed_params,
            named_params,
        };

        new_fn.verify_main_fn()?;
        Ok(new_fn)
    }
}
impl FuncSignature {
    pub fn with_mangled_name(self) -> Self {
        Self {
            named_params: self.named_params,
            pos_params: self.pos_params,
            return_type: self.return_type,
            name: mangle_function_name(&self.name, None),
        }
    }

    ///
    /// Convert this function signature to a method
    /// This works by adding an parameter of type struct_name to the beginning of this func's arguments
    ///
    pub(super) fn method(self, struct_name: Ident) -> Self {
        let new_name = mangle_function_name(&self.name, Some(&struct_name));
        let mut new_unnamed_params = vec![PosParam {
            name: Ident::from("self"),
            crab_type: CrabType::STRUCT(struct_name),
        }];
        new_unnamed_params.extend(self.pos_params);
        Self {
            name: new_name,
            return_type: self.return_type,
            named_params: self.named_params,
            pos_params: new_unnamed_params,
        }
    }

    fn verify_main_fn(&self) -> Result<bool> {
        if self.name == main_func_name() {
            if self.return_type != CrabType::STRUCT(int_struct_name())
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

struct PosParams(Vec<PosParam>);
try_from_pair!(PosParams, Rule::pos_params);
impl AstNode for PosParams {
    fn from_pair(pair: Pair<Rule>) -> Result<Self>
        where
            Self: Sized,
    {
        Ok(Self(
            pair.into_inner().try_fold(vec![], |params, param| {
                Result::Ok(fields.fpush(PosParam::try_from(field)?))
            })?,
        ))
    }
}

struct NamedParams(Vec<NamedParam>);
try_from_pair!(NamedParams, Rule::named_params);
impl AstNode for NamedParams {
    fn from_pair(pair: Pair<Rule>) -> Result<Self>
        where
            Self: Sized,
    {
        Ok(Self(
            pair.into_inner().try_fold(vec![], |params, param| {
                Result::Ok(fields.fpush(NamedParam::try_from(field)?))
            })?,
        ))
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
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

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NamedParam {
    pub name: Ident,
    pub crab_type: CrabType,
    pub expr: Expression,
}
try_from_pair!(NamedParam, Rule::named_fn_param);
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
