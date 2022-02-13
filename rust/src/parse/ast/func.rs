use crate::parse::ast::{AstNode, CodeBlock, CrabType, Expression, Ident, int_struct_name, internal_main_func_name, main_func_name, Statement};
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
}

#[derive(Debug, Clone)]
pub struct FuncSignature {
    pub name: Ident,
    pub return_type: CrabType,
    pub unnamed_params: Vec<FnParam>,
    pub named_params: Vec<NamedFnParam>,
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
                    unnamed_params.push(FnParam::try_from(inner_pair)?);
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
                    named_params.push(NamedFnParam::try_from(inner_pair)?);
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

        let mut new_fn = Self {
            name,
            return_type,
            unnamed_params,
            named_params,
        };

        let is_main = new_fn.verify_main_fn()?;
        if is_main {
            new_fn.name = internal_main_func_name();
        }

        Ok(new_fn)
    }
}
impl FuncSignature {
    pub fn get_params(&self) -> Vec<FnParam> {
        let mut params = vec![];
        for param in &self.unnamed_params {
            params.push(param.clone())
        }
        for named_param in &self.named_params {
            params.push(FnParam {
                name: named_param.name.clone(),
                crab_type: named_param.crab_type.clone(),
            })
        }
        params
    }

    ///
    /// Convert this function signature to a method
    /// This works by adding an parameter of type struct_name to the beginning of this func's arguments
    ///
    fn method(self, struct_name: Ident) -> Self {
        let mut unnamed_params = vec![FnParam {
            name: Ident::from("self"),
            crab_type: CrabType::STRUCT(struct_name),
        }];
        unnamed_params.extend(self.unnamed_params);
        Self {
            name: self.name,
            return_type: self.return_type,
            named_params: self.named_params,
            unnamed_params: unnamed_params,
        }
    }

    fn verify_main_fn(&self) -> Result<bool> {
        if self.name == main_func_name() {
            if self.return_type != CrabType::STRUCT(int_struct_name()) || !self.unnamed_params.is_empty() || !self.named_params.is_empty() {
                Err(ParseError::MainSignature)
            } else {
                Ok(true)
            }
        } else {
            Ok(false)
        }
    }
}

#[derive(Debug, Clone)]
pub struct FnParam {
    pub name: Ident,
    pub crab_type: CrabType,
}

try_from_pair!(FnParam, Rule::fn_param);
impl AstNode for FnParam {
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

#[derive(Debug, Clone)]
pub struct NamedFnParam {
    pub name: Ident,
    pub crab_type: CrabType,
    pub expr: Expression,
}

try_from_pair!(NamedFnParam, Rule::named_fn_param);
impl AstNode for NamedFnParam {
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

#[derive(Debug, Clone)]
pub struct FnCall {
    pub name: Ident,
    pub unnamed_args: Vec<Expression>,
    pub named_args: Vec<NamedExpression>,
}

try_from_pair!(FnCall, Rule::fn_call);
impl AstNode for FnCall {
    fn from_pair(pair: Pair<Rule>) -> Result<Self>
    where
        Self: Sized,
    {
        let mut inner = pair.into_inner();
        let name = Ident::from(inner.next().ok_or(ParseError::ExpectedInner)?.as_str());
        let mut named_args = vec![];
        let mut unnamed_args = vec![];
        let mut seen_named_arg = false;

        for inner_pair in inner {
            match inner_pair.clone().as_rule() {
                Rule::expression => {
                    if seen_named_arg {
                        return Err(ParseError::PositionalArgAfterNamedParam(name.clone()));
                    }
                    unnamed_args.push(Expression::try_from(inner_pair)?);
                }
                Rule::named_expression => {
                    named_args.push(NamedExpression::try_from(inner_pair)?);
                    seen_named_arg = true;
                }
                _ => return Err(ParseError::NoMatch(String::from("FnCall::from_pair"))),
            }
        }

        Ok(Self {
            name,
            named_args,
            unnamed_args,
        })
    }
}

#[derive(Debug, Clone)]
pub struct NamedExpression {
    pub name: Ident,
    pub expr: Expression,
}

try_from_pair!(NamedExpression, Rule::named_expression);
impl AstNode for NamedExpression {
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
