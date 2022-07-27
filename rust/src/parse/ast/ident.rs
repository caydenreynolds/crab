use std::collections::HashMap;
use std::convert::TryFrom;
use std::fmt::{Display, Formatter};
use pest::iterators::Pair;
use crate::compile::CompileError;
use crate::{compile, parse, try_from_pair};
use crate::parse::ast::{AstNode};
use crate::parse::{ParseError, Rule};
use crate::util::{ListFunctional, MapFunctional};

pub type Ident = String;

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum CrabType {
    VOID,
    PRIM_INT,
    PRIM_STR,
    PRIM_BOOL,
    SIMPLE(Ident),
    LIST(Box<CrabType>),
    TMPL(Ident, Vec<CrabType>),
}
try_from_pair!(CrabType, Rule::crab_type);
impl AstNode for CrabType {
    fn from_pair(pair: Pair<Rule>) -> parse::Result<Self>
        where
            Self: Sized,
    {
        let next = pair.into_inner().next().ok_or(ParseError::ExpectedInner)?;
        match next.as_rule() {
            Rule::simple_crab_type => Ok(Self::SIMPLE(SimpleCrabType::try_from(next)?.0)),
            Rule::list_crab_type => Ok(Self::LIST(ListCrabType::try_from(next)?.0)),
            Rule::tmpl_crab_type => {
                let tct = TmplCrabType::try_from(next)?;
                Ok(Self::TMPL(tct.0, tct.1))
            },
            _ => Err(ParseError::NoMatch(String::from("CrabType::from_pair"))),
        }
    }
}
impl CrabType {
    pub fn try_get_struct_name(&self) -> compile::Result<Ident> {
        match self {
            Self::SIMPLE(id) => Ok(id.clone()),
            Self::TMPL(id, _) => Ok(id.clone()),
            Self::LIST(ct) => Ok(ct.try_get_struct_name()?),
            _ => Err(CompileError::NotAStruct(
                StructId::from_name(Ident::from("unknown")),
                String::from("CrabType::try_get_struct_name"),
            )),
        }
    }

    pub(super) fn resolve(self, caller_id: &StructId, tmpls: &[CrabType]) -> compile::Result<Self> {
        let resolution_map = caller_id
            .tmpls
            .iter()
            .zip(tmpls.iter())
            .fold(HashMap::new(), |resolution_map, (caller_tmpl, real_tmpl)| {
                resolution_map.finsert(caller_tmpl.clone(), real_tmpl.clone())
            });
        match self {
            CrabType::SIMPLE(name) => {
                match resolution_map.get(&StructId::from_name(name)) {
                    None => Ok(Self),
                    Some(ct) => Ok(ct.clone()),
                }
            }
            CrabType::LIST(ct) => Ok(CrabType::LIST(Box::new(ct.resolve(caller_id, tmpls)?))),
            CrabType::TMPL(name, inner_tmpls) => {
                let name = match resolution_map.get(&StructId::from_name(name.clone())) {
                    None => name,
                    Some(ct) => ct.try_get_struct_name()?,
                };
                let resolved_tmpls = inner_tmpls
                    .into_iter()
                    .try_fold(vec![], |resolved_tmpls, inner_tmpl| {
                        Ok(resolved_tmpls.fpush(inner_tmpl.resolve(caller_id, tmpls)?))
                    })?;
                Ok(CrabType::TMPL(name, resolved_tmpls))
            }
            _ => Ok(Self),
        }
    }
}
impl Display for CrabType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CrabType::VOID => write!(f, "VOID"),
            CrabType::PRIM_BOOL => write!(f, "PRIM_BOOL"),
            CrabType::PRIM_STR => write!(f, "PRIM_STR"),
            CrabType::PRIM_INT => write!(f, "PRIM_INT"),
            CrabType::SIMPLE(n) => write!(f, "{}", n),
            CrabType::LIST(l) => write!(f, "LIST_{}", l),
            CrabType::TMPL(i, t) => {
                write!(f, "TMPL_{}", i)?;
                for ct in t {
                    write!(f, "_{}", ct)?;
                }
                Ok(())
            }
        }
    }
}
impl From<StructId> for CrabType {
    fn from(si: StructId) -> Self {
        if si.tmpls.is_empty() {
            CrabType::SIMPLE(si.name)
        } else {
            CrabType::TMPL(si.name, si.tmpls.into_iter().map(|tmpl| tmpl.into()).collect())
        }
    }
}

struct SimpleCrabType(Ident);
try_from_pair!(SimpleCrabType, Rule::simple_crab_type);
impl AstNode for SimpleCrabType {
    fn from_pair(pair: Pair<Rule>) -> parse::Result<Self> where Self: Sized {
        Ok(Self(Ident::from(pair.into_inner().next().ok_or(ParseError::ExpectedInner)?.as_str())))
    }
}

struct ListCrabType(Box<CrabType>);
try_from_pair!(ListCrabType, Rule::list_crab_type);
impl AstNode for ListCrabType {
    fn from_pair(pair: Pair<Rule>) -> parse::Result<Self> where Self: Sized {
        Ok(Self(Box::new(CrabType::try_from(pair.into_inner().next().ok_or(ParseError::ExpectedInner)?)?)))
    }
}

struct TmplCrabType(Ident, Vec<CrabType>);
try_from_pair!(TmplCrabType, Rule::tmpl_crab_type);
impl AstNode for TmplCrabType {
    fn from_pair(pair: Pair<Rule>) -> parse::Result<Self> where Self: Sized {
        let mut inner = pair.into_inner();
        let name = Ident::from(inner.next().ok_or(ParseError::ExpectedInner)?.as_str());
        let tmpls = inner.try_fold(vec![], |tmpls, tmpl| {
            parse::Result::Ok(tmpls.fpush(CrabType::try_from(tmpl)?))
        })?;
        Ok(Self(name, tmpls))
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct StructId {
    pub name: Ident,
    pub tmpls: Vec<StructId>,
}
try_from_pair!(StructId, Rule::struct_id);
impl AstNode for StructId {
    fn from_pair(pair: Pair<Rule>) -> parse::Result<Self> where Self: Sized {
        let mut inner = pair.into_inner();
        let name = Ident::from(inner.next().ok_or(ParseError::ExpectedInner)?.as_str());
        let tmpls = inner.fold(vec![], |tmpls, tmpl| {
            tmpls.fpush(StructId::from_name(Ident::from(tmpl.as_str())))
        });
        Ok(Self {
            name,
            tmpls,
        })
    }
}
impl StructId {
    /// Returns the mangled name for this StructId
    /// NOTE: This mangling algorithm is quick, dirty, and terrible. It can and will cause name collisions
    pub fn mangle(&self) -> String {
        let mangled = format!("_STRUCT_{}", self.name);
        let mangled = self.tmpls.iter().fold(mangled, |mangled, tmpl| {
            format!("{}_{}", mangled, tmpl.mangle())
        });
        mangled
    }

    /// Returns a StructId that has the given name and no tmpls
    pub fn from_name(name: Ident) -> Self {
        Self {
            name,
            tmpls: vec![],
        }
    }

    /// Consumes self, returning a StructId with the tmpl types resolved according to the
    /// given slice of CrabTypes
    pub fn resolve(self, types: &[CrabType]) -> compile::Result<Self> {
        if types.len() != self.tmpls.len() {
            return Err(CompileError::WrongTemplateTypeCount(self.tmpls.len(), types.len()))
        }
        // else
        self.add_tmpls(types)
    }

    /// Consume this StructId, returning a new one with the tmpls set to match all of the given types
    fn add_tmpls(self, types: &[CrabType]) -> compile::Result<Self> {
        Ok(Self {
            tmpls: types.iter().try_fold(vec![], |tmpls, ct| {
                match ct {
                    CrabType::VOID => Err(CompileError::VoidType),
                    CrabType::PRIM_INT | CrabType::PRIM_STR | CrabType::PRIM_BOOL => {
                        Err(CompileError::NotAStruct(
                            StructId::from_name(format!("{}", ct)),
                            String::from("StructId::add_tmpls")
                        ))
                    },
                    CrabType::SIMPLE(n) => {
                        Ok(tmpls.fpush(StructId::from_name(n.clone())))
                    },
                    CrabType::LIST(_) => todo!(),
                    CrabType::TMPL(n, ct_tmpls) => {
                        Ok(tmpls.fpush(StructId::from_name(n.clone()).add_tmpls(ct_tmpls)?))
                    },
                }
            })?,
            ..self
        })
    }
}
impl Display for StructId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "STRUCT_{}", self.name)?;
        self.tmpls.iter().try_for_each(|tmpl| {
            write!(f, "_{}", tmpl)
        })
    }
}
impl TryFrom<CrabType> for StructId {
    type Error = CompileError;

    fn try_from(ct: CrabType) -> std::result::Result<Self, Self::Error> {
        match ct {
            CrabType::SIMPLE(name) => Ok(Self { name, tmpls: vec![] }),
            CrabType::LIST(inner) => StructId::try_from(*inner),
            CrabType::TMPL(name, tmpls) => {
                Ok(Self {
                    name,
                    tmpls: tmpls.into_iter().try_fold(vec![], |tmpls, tmpl| {
                        compile::Result::Ok(tmpls.fpush(StructId::try_from(tmpl)?))
                    })?
                })
            }
            _ => Err(CompileError::NotAStruct(StructId::from_name(format!("{}", ct)), String::from("CrabType::try_from"))),
        }
    }
}
