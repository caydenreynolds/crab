use crate::parse::ast::{AstNode, CrabInterface, Func, Ident, Struct, StructImpl, StructIntr};
use crate::parse::{ParseError, Result, Rule};
use crate::try_from_pair;
use pest::iterators::Pair;
use std::collections::HashMap;
use std::convert::TryFrom;

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct CrabAst {
    pub functions: Vec<Func>,
    pub structs: Vec<Struct>,
    pub interfaces: HashMap<Ident, CrabInterface>,

    impls: Vec<StructImpl>,
    intrs: Vec<StructIntr>,
}

try_from_pair!(CrabAst, Rule::program);
impl AstNode for CrabAst {
    fn from_pair(pair: Pair<Rule>) -> Result<Self> {
        let inner = pair.into_inner();
        let mut functions = vec![];
        let mut structs = vec![];
        let mut impls = vec![];
        let mut interfaces = HashMap::new();
        let mut intrs = vec![];

        for in_pair in inner {
            match in_pair.clone().as_rule() {
                Rule::function => functions.push(Func::try_from(in_pair)?.with_mangled_name()),
                Rule::crab_struct => structs.push(Struct::try_from(in_pair)?),
                Rule::impl_block => impls.push(StructImpl::try_from(in_pair)?),
                Rule::interface => {
                    let interface = CrabInterface::try_from(in_pair)?;
                    interfaces.insert(interface.name.clone(), interface);
                }
                Rule::intr_block => intrs.push(StructIntr::try_from(in_pair)?),
                Rule::EOI => break, // Nothing should ever show up after EOI
                _ => return Err(ParseError::NoMatch(String::from("CrabAst::from_pair"))),
            }
        }

        for struct_impl in &impls {
            for func in &struct_impl.fns {
                functions.push(func.clone().method(struct_impl.struct_name.clone()));
            }
        }

        Ok(Self {
            functions,
            structs,
            interfaces,
            intrs,
            impls,
        })
    }
}
impl CrabAst {
    pub fn join(self, other: Self) -> Self {
        Self {
            impls: self.impls.into_iter().chain(other.impls.into_iter()).collect(),
            functions: self.functions.into_iter().chain(other.functions.into_iter()).collect(),
            structs: self.structs.into_iter().chain(other.structs.into_iter()).collect(),
            interfaces: self.interfaces.into_iter().chain(other.interfaces.into_iter()).collect(),
            intrs: self.intrs.into_iter().chain(other.intrs.into_iter()).collect(),
        }
    }
    pub fn verify(&self) -> Result<()> {
        self.verify_intrs()
    }

    fn verify_intrs(&self) -> Result<()> {
        for intr in &self.intrs {
            for si in &self.impls {
                if si.struct_name == intr.struct_name {
                    for inter in &intr.inters {
                        si.verify_implements(
                            self.interfaces
                                .get(inter)
                                .ok_or(ParseError::InterfaceNotFound(inter.clone()))?,
                        )?;
                    }
                    break;
                } else {
                    // Do nothing
                }
            }
        }
        Ok(())
    }
}
