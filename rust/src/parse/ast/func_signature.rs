use crate::parse::ast::{AstNode, CrabType, FnParam, Ident, NamedFnParam};
use crate::parse::{ParseError, Result, Rule};
use crate::try_from_pair;
use crate::util::{int_struct_name, main_func_name, mangle_function_name};
use pest::iterators::Pair;
use std::convert::TryFrom;

#[derive(Debug, Clone, Eq, PartialEq)]
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

        let new_fn = Self {
            name,
            return_type,
            unnamed_params,
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
            unnamed_params: self.unnamed_params,
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
        let mut new_unnamed_params = vec![FnParam {
            name: Ident::from("self"),
            crab_type: CrabType::STRUCT(struct_name),
        }];
        new_unnamed_params.extend(self.unnamed_params);
        Self {
            name: new_name,
            return_type: self.return_type,
            named_params: self.named_params,
            unnamed_params: new_unnamed_params,
        }
    }

    fn verify_main_fn(&self) -> Result<bool> {
        if self.name == main_func_name() {
            if self.return_type != CrabType::STRUCT(int_struct_name())
                || !self.unnamed_params.is_empty()
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
