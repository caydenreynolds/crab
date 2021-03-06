use crate::parse::ast::{
    AstNode, CrabInterface, CrabStruct, Func, Ident, StructId, StructImpl, StructIntr,
};
use crate::parse::{ParseError, Result, Rule};
use crate::try_from_pair;
use crate::util::main_func_name;
use pest::iterators::Pair;
use std::collections::HashMap;
use std::convert::TryFrom;

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct CrabAst {
    pub functions: HashMap<Ident, Func>,
    pub structs: Vec<CrabStruct>,
    pub interfaces: HashMap<Ident, CrabInterface>,
    pub main: Option<Func>,
    pub intrs: Vec<StructIntr>,
    pub impls: HashMap<StructId, StructImpl>,
}

try_from_pair!(CrabAst, Rule::program);
impl AstNode for CrabAst {
    fn from_pair(pair: Pair<Rule>) -> Result<Self> {
        let inner = pair.into_inner();
        let mut functions = HashMap::new();
        let mut structs = vec![];
        let mut impls = HashMap::new();
        let mut interfaces = HashMap::new();
        let mut intrs = vec![];
        let mut main = None;

        for in_pair in inner {
            match in_pair.clone().as_rule() {
                Rule::function => {
                    let func = Func::try_from(in_pair)?;
                    if func.signature.name == main_func_name() {
                        main = Some(func.clone());
                    }
                    functions.insert(func.signature.name.clone(), func);
                }
                Rule::crab_struct => structs.push(CrabStruct::try_from(in_pair)?),
                Rule::impl_block => {
                    let struct_impl = StructImpl::try_from(in_pair)?;
                    impls.insert(struct_impl.struct_id.clone(), struct_impl);
                }
                Rule::interface => {
                    let interface = CrabInterface::try_from(in_pair)?;
                    interfaces.insert(interface.name.clone(), interface);
                }
                Rule::intr_block => intrs.push(StructIntr::try_from(in_pair)?),
                Rule::EOI => break, // Nothing should ever show up after EOI
                _ => return Err(ParseError::NoMatch(String::from("CrabAst::from_pair"))),
            }
        }

        Ok(Self {
            functions,
            structs,
            interfaces,
            intrs,
            impls,
            main,
        })
    }
}
impl CrabAst {
    pub fn join(self, other: Self) -> Self {
        Self {
            impls: self
                .impls
                .into_iter()
                .chain(other.impls.into_iter())
                .collect(),
            functions: self
                .functions
                .into_iter()
                .chain(other.functions.into_iter())
                .collect(),
            structs: self
                .structs
                .into_iter()
                .chain(other.structs.into_iter())
                .collect(),
            interfaces: self
                .interfaces
                .into_iter()
                .chain(other.interfaces.into_iter())
                .collect(),
            intrs: self
                .intrs
                .into_iter()
                .chain(other.intrs.into_iter())
                .collect(),
            main: self.main.or(other.main),
        }
    }
    pub fn verify(&self) -> Result<()> {
        self.verify_intrs()
    }

    fn verify_intrs(&self) -> Result<()> {
        for intr in &self.intrs {
            for (sid, simp) in &self.impls {
                if *sid == intr.struct_id {
                    for inter in &intr.inters {
                        simp.verify_implements(
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
