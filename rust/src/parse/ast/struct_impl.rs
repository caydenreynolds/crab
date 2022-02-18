use crate::parse::ast::{AstNode, CrabInterface, Func, Ident};
use crate::parse::{ParseError, Result, Rule};
use crate::try_from_pair;
use pest::iterators::Pair;
use std::convert::TryFrom;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct StructImpl {
    pub struct_name: Ident,
    pub interface_name: Option<Ident>,
    pub fns: Vec<Func>,
}

try_from_pair!(StructImpl, Rule::impl_block);
impl AstNode for StructImpl {
    fn from_pair(pair: Pair<Rule>) -> Result<Self>
    where
        Self: Sized,
    {
        let mut inner = pair.into_inner();
        let struct_name = Ident::from(inner.next().ok_or(ParseError::ExpectedInner)?.as_str());
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

        let mut fns = vec![];
        for in_pair in inner {
            fns.push(Func::try_from(in_pair)?);
        }

        Ok(Self {
            struct_name,
            interface_name,
            fns,
        })
    }
}
impl StructImpl {
    pub fn verify_implements(&self, intr: &CrabInterface) -> Result<()> {
        for ifunc in &intr.fns {
            let mut match_found = false;
            for func in &self.fns {
                if func.signature == *ifunc {
                    match_found = true;
                }
            }
            if !match_found {
                return Err(ParseError::DoesNotImplement(
                    self.struct_name.clone(),
                    ifunc.name.clone(),
                    intr.name.clone(),
                ));
            }
        }
        Ok(())
    }
}
