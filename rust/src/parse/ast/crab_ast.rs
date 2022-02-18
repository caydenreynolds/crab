use crate::parse::ast::{AstNode, CrabInterface, Func, Ident, Struct, StructImpl, StructIntr};
use crate::parse::{ParseError, Result, Rule};
use crate::try_from_pair;
use pest::iterators::Pair;
use std::collections::HashMap;
use std::convert::TryFrom;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CrabAst {
    pub functions: Vec<Func>,
    pub structs: Vec<Struct>,
    pub interfaces: HashMap<Ident, CrabInterface>,
}

try_from_pair!(CrabAst, Rule::program);
impl AstNode for CrabAst {
    fn from_pair(pair: Pair<Rule>) -> Result<Self> {
        let inner = pair.into_inner();
        let mut functions = vec![];
        let mut structs = vec![];
        let mut impls = vec![];
        let mut interfaces = HashMap::new();
        let mut struct_intrs = vec![];

        for in_pair in inner {
            match in_pair.clone().as_rule() {
                Rule::function => functions.push(Func::try_from(in_pair)?.with_mangled_name()),
                Rule::crab_struct => structs.push(Struct::try_from(in_pair)?),
                Rule::impl_block => impls.push(StructImpl::try_from(in_pair)?),
                Rule::interface => {
                    let interface = CrabInterface::try_from(in_pair)?;
                    interfaces.insert(interface.name.clone(), interface);
                }
                Rule::intr_block => struct_intrs.push(StructIntr::try_from(in_pair)?),
                Rule::EOI => break, // Nothing should ever show up after EOI
                _ => return Err(ParseError::NoMatch(String::from("CrabAst::from_pair"))),
            }
        }

        Self::verify_intrs(struct_intrs, &impls, &interfaces)?;

        for struct_impl in impls {
            for func in struct_impl.fns {
                functions.push(func.method(struct_impl.struct_name.clone()));
            }
        }

        Ok(Self {
            functions,
            structs,
            interfaces,
        })
    }
}
impl CrabAst {
    fn verify_intrs(
        intrs: Vec<StructIntr>,
        impls: &Vec<StructImpl>,
        interfaces: &HashMap<Ident, CrabInterface>,
    ) -> Result<()> {
        for intr in intrs {
            for si in impls {
                if si.struct_name == intr.struct_name {
                    for inter in intr.inters {
                        si.verify_implements(
                            interfaces
                                .get(&inter)
                                .ok_or(ParseError::InterfaceNotFound(inter))?,
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
