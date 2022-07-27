use crate::parse::ast::{AstNode, CrabInterface, Func, Ident, StructId};
use crate::parse::{ParseError, Result, Rule};
use crate::try_from_pair;
use crate::util::{ListFunctional, MapFunctional};
use pest::iterators::Pair;
use std::collections::HashMap;
use std::convert::TryFrom;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct StructImpl {
    pub struct_id: StructId,
    pub interface_name: Option<Ident>,
    pub fns: HashMap<Ident, Func>,
}
try_from_pair!(StructImpl, Rule::impl_block);
impl AstNode for StructImpl {
    fn from_pair(pair: Pair<Rule>) -> Result<Self>
    where
        Self: Sized,
    {
        let mut inner = pair.into_inner();
        let struct_id = StructId::try_from(inner.next().ok_or(ParseError::ExpectedInner)?)?;

        let next_opt = inner.peek();
        let interface_name = match next_opt {
            None => None,
            Some(next_pair) => match next_pair.clone().as_rule() {
                Rule::function => None,
                Rule::ident => {
                    inner.next();
                    Some(Ident::from(next_pair.as_str()))
                }
                rule => {
                    return Err(ParseError::IncorrectRule(
                        String::from("StructImpl"),
                        String::from("function or ident"),
                        format!("{:#?}", rule),
                    ))
                }
            },
        };

        let fns = inner.try_fold(HashMap::new(), |fns, func| {
            let f = Func::try_from(func)?.method(struct_id.clone());
            Result::Ok(fns.finsert(f.signature.name.clone(), f))
        })?;

        Ok(Self {
            struct_id,
            interface_name,
            fns,
        })
    }
}
impl StructImpl {
    pub fn verify_implements(&self, intr: &CrabInterface) -> Result<()> {
        for ifunc in &intr.fns {
            let mut match_found = false;
            for (_, func) in &self.fns {
                if func.signature.implements(ifunc) {
                    match_found = true;
                    break;
                }
            }
            if !match_found {
                return Err(ParseError::DoesNotImplement(
                    self.struct_id.clone(),
                    ifunc.name.clone(),
                    intr.name.clone(),
                ));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct StructIntr {
    pub struct_id: StructId,
    pub inters: Vec<Ident>,
}
try_from_pair!(StructIntr, Rule::intr_block);
impl AstNode for StructIntr {
    fn from_pair(pair: Pair<Rule>) -> Result<Self>
    where
        Self: Sized,
    {
        let mut inner = pair.into_inner();
        let struct_name = StructId::try_from(inner.next().ok_or(ParseError::ExpectedInner)?)?;

        let inters = inner.fold(vec![], |inters, inter| {
            inters.fpush(Ident::from(inter.as_str()))
        });

        Ok(Self {
            struct_id: struct_name,
            inters,
        })
    }
}
