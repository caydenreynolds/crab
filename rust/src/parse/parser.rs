use crate::compile;
use crate::compile::{AstVisitor, CompileError};
use crate::parse::{parse_string, ParseError, Result};
use inkwell::context::Context;
use inkwell::types::{AnyTypeEnum, BasicMetadataTypeEnum, BasicTypeEnum, FunctionType};
use inkwell::AddressSpace;
use log::trace;
use pest::{iterators::Pair, Parser};
use std::convert::TryFrom;
use std::fs;
use std::path::Path;

#[derive(pest_derive::Parser)]
#[grammar = "parse/grammar.pest"]
struct CrabParser;

pub fn parse(source: &Path) -> Result<CrabAst> {
    let source = fs::read_to_string(source)?;
    let parsed = CrabParser::parse(Rule::program, &source)?;
    trace!("Parsed source is: {:#?}", parsed);
    // // There can only be one
    return match parsed.peek() {
        None => Err(ParseError::NoMatch(String::from("parse"))),
        Some(pair) => CrabAst::try_from(pair),
    };
}

/*
 *******************************************************************************
 *                                                                             *
 *                                STRUCTS                                      *
 *                                                                             *
 *******************************************************************************
*/

#[derive(Debug, Clone)]
pub struct CrabAst {
    pub functions: Vec<Func>,
}

#[derive(Debug, Clone)]
pub struct Func {
    pub signature: FuncSignature,
    pub body: CodeBlock,
}

#[derive(Debug, Clone)]
pub struct FuncSignature {
    pub name: Ident,
    pub return_type: CrabType,
    pub args: Option<TypedIdentList>,
}

pub type Ident = String;

#[derive(Debug, Clone)]
pub struct TypedIdent {
    pub name: Ident,
    pub crab_type: CrabType,
}

#[derive(Debug, Clone)]
pub struct IdentList {
    pub idents: Vec<Ident>,
}

#[derive(Debug, Clone)]
pub struct TypedIdentList {
    pub typed_idents: Vec<TypedIdent>,
}

#[derive(Debug, Clone)]
pub struct ExpressionList {
    pub expressions: Vec<Expression>,
}

#[derive(Debug, Clone)]
pub struct CodeBlock {
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub struct Statement {
    pub expression: Option<Expression>,
    pub statement_type: StatementType,
}

#[derive(Debug, Clone)]
#[allow(non_camel_case_types)]
pub enum StatementType {
    RETURN,
    ASSIGNMENT(Assignment),
    REASSIGNMENT(Assignment),
    FN_CALL(FnCall),
    IF_STATEMENT(IfStmt),
    WHILE_STATEMENT(WhileStmt),
    DO_WHILE_STATEMENT(DoWhileStmt),
}

#[derive(Debug, Clone)]
pub struct Assignment {
    pub var_name: Ident,
}

#[derive(Debug, Clone)]
pub struct FnCall {
    pub name: Ident,
    pub args: ExpressionList,
}

#[derive(Debug, Clone)]
pub struct IfStmt {
    pub expr: Expression,
    pub then: CodeBlock,
    pub else_stmt: Option<Box<ElseStmt>>,
}

#[derive(Debug, Clone)]
pub enum ElseStmt {
    ELSE(CodeBlock),
    ELIF(IfStmt),
}

#[derive(Debug, Clone)]
pub struct WhileStmt {
    pub expr: Expression,
    pub then: CodeBlock,
}

#[derive(Debug, Clone)]
pub struct DoWhileStmt {
    pub expr: Expression,
    pub then: CodeBlock,
}

#[derive(Debug, Clone)]
#[allow(non_camel_case_types)]
pub enum Expression {
    PRIM(Primitive),
    FN_CALL(FnCall),
    VARIABLE(Ident),
}

#[derive(Debug, Clone)]
pub enum Primitive {
    UINT(u64),
    STRING(String),
    BOOL(bool),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum CrabType {
    UINT,
    VOID,
    STRING,
    FLOAT,
    BOOL,
}

/*
 *******************************************************************************
 *                                                                             *
 *                              TRAITS 'N STUFF                                *
 *                                                                             *
 *******************************************************************************
*/

pub trait AstNode {
    // Build an instance of self from the given pair, assuming that the pair's rule type is correct
    fn from_pair(pair: Pair<Rule>) -> Result<Self>
    where
        Self: Sized;
    fn pre_visit(&self, visitor: &mut dyn AstVisitor) -> compile::Result<()>;
    fn visit(&self, visitor: &mut dyn AstVisitor) -> compile::Result<()>;
    fn post_visit(&self, visitor: &mut dyn AstVisitor) -> compile::Result<()>;
}

/*
 * Generates a try-from function for getting the given crabNode from the specified rule type
 * The crabNode must implement NodeFromPair for the given rule type
 *
 * The first form only matches the given rule
 * The second form matches the given rule and ignores the second supplied rule
 * TODO: The third form matches the given rule and ignores all subsequent rules
 */
macro_rules! try_from_pair {
    ($crabNode:ty, $rule:path) => {
        impl TryFrom<Pair<'_, Rule>> for $crabNode {
            type Error = ParseError;
            fn try_from(pair: Pair<Rule>) -> std::result::Result<$crabNode, Self::Error> {
                match pair.as_rule() {
                    $rule => <$crabNode>::from_pair(pair),
                    _ => Err(ParseError::IncorrectRule(
                        String::from(stringify!($crabNode)),
                        String::from(stringify!($rule)),
                        format!("{:?}", pair.as_rule()),
                    )),
                }
            }
        }
    };

    ($crabNode:ty, $rule:path, $ig_rule:path) => {
        impl TryFrom<Pair<'_, Rule>> for $crabNode {
            type Error = ParseError;
            fn try_from(pair: Pair<Rule>) -> std::result::Result<$crabNode, Self::Error> {
                match pair.as_rule() {
                    $rule => <$crabNode>::from_pair(pair),
                    $ig_rule => Err(ParseError::None),
                    _ => Err(ParseError::IncorrectRule(
                        String::from(stringify!($crabNode)),
                        String::from(stringify!($rule)),
                        format!("{:?}", pair.as_rule()),
                    )),
                }
            }
        }
    };
}

macro_rules! visit_fns {
    ($node:ty) => {
        paste::item! {
            fn pre_visit(&self, visitor: &mut dyn AstVisitor) -> compile::Result<()> {
                visitor.[< pre_visit_ $node >](self)
            }

            fn visit(&self, visitor: &mut dyn AstVisitor) -> compile::Result<()> {
                visitor.[< visit_ $node >](self)
            }

            fn post_visit(&self, visitor: &mut dyn AstVisitor) -> compile::Result<()> {
                visitor.[< post_visit_ $node >](self)
            }
        }
    };
}

/*
 *******************************************************************************
 *                                                                             *
 *                               BUILD AST                                     *
 *                                                                             *
 *******************************************************************************
*/

try_from_pair!(CrabAst, Rule::program);
impl AstNode for CrabAst {
    fn from_pair(pair: Pair<Rule>) -> Result<Self> {
        let mut functions = vec![];
        for function in pair.into_inner() {
            match Func::try_from(function) {
                Ok(func) => functions.push(func),
                Err(err) => {
                    match err {
                        ParseError::None => {} // Do nothing
                        _ => return Err(err),
                    }
                }
            }
        }
        Ok(CrabAst { functions })
    }

    visit_fns!(CrabAst);
}

try_from_pair!(Func, Rule::function, Rule::EOI);
impl AstNode for Func {
    fn from_pair(pair: Pair<Rule>) -> Result<Self> {
        let mut inner = pair.into_inner();
        let sig_pair = inner.next().ok_or(ParseError::ExpectedInner)?;
        let signature = FuncSignature::try_from(sig_pair)?;
        let body_pair = inner.next().ok_or(ParseError::ExpectedInner)?;
        let body = CodeBlock::try_from(body_pair)?;

        Ok(Func { signature, body })
    }

    visit_fns!(Func);
}

try_from_pair!(FuncSignature, Rule::fn_signature);
impl AstNode for FuncSignature {
    fn from_pair(pair: Pair<Rule>) -> Result<Self> {
        let mut inner = pair.into_inner();
        let name = Ident::from(inner.next().ok_or(ParseError::ExpectedInner)?.as_str());

        let (args, return_type) = match inner.clone().count() {
            0 => (None, CrabType::VOID),
            1 => {
                // Determine whether we have args or a return
                let in_pair = inner.next().ok_or(ParseError::None)?;
                trace!("{:#?}", in_pair.clone().as_rule());
                match in_pair.clone().as_rule() {
                    // We have args
                    Rule::typed_ident_list => {
                        (Some(TypedIdentList::try_from(in_pair)?), CrabType::VOID)
                    }
                    // We have a return
                    Rule::crab_type => (None, CrabType::try_from(in_pair)?),
                    // This should never happen
                    _ => {
                        return Err(ParseError::NoMatch(String::from(
                            "FuncSignature::from_pair_0",
                        )))
                    }
                }
            }
            2 => (
                Some(TypedIdentList::try_from(
                    inner.next().ok_or(ParseError::ExpectedInner)?,
                )?),
                match inner.next() {
                    Some(return_pair) => CrabType::try_from(return_pair)?,
                    None => CrabType::VOID,
                },
            ),
            _ => {
                return Err(ParseError::NoMatch(String::from(
                    "FuncSignature::from_pair_1",
                )))
            }
        };

        Ok(FuncSignature {
            name,
            return_type,
            args,
        })
    }

    visit_fns!(FuncSignature);
}

impl FuncSignature {
    pub fn get_args(&self) -> &[TypedIdent] {
        return match &self.args {
            Some(ident_list) => ident_list.typed_idents.as_slice(),
            None => &[],
        };
    }
}

try_from_pair!(CodeBlock, Rule::code_block);
impl AstNode for CodeBlock {
    fn from_pair(pair: Pair<Rule>) -> Result<Self> {
        let mut statements = vec![];
        for statement in pair.into_inner() {
            match Statement::try_from(statement) {
                Ok(stmt) => statements.push(stmt),
                Err(err) => {
                    match err {
                        ParseError::None => {} // Do nothing
                        _ => return Err(err),
                    }
                }
            }
        }
        Ok(CodeBlock { statements })
    }

    visit_fns!(CodeBlock);
}

try_from_pair!(Statement, Rule::statement);
impl AstNode for Statement {
    fn from_pair(pair: Pair<Rule>) -> Result<Self> {
        let statement_type = StatementType::try_from(pair.clone())?;
        let expression = statement_type.get_expression(pair)?;

        Ok(Self {
            statement_type,
            expression,
        })
    }

    visit_fns!(Statement);
}

try_from_pair!(StatementType, Rule::statement);
#[allow(unreachable_patterns)]
impl AstNode for StatementType {
    fn from_pair(pair: Pair<Rule>) -> Result<Self> {
        let mut inner = pair.into_inner();
        let expr_type = inner.next().ok_or(ParseError::ExpectedInner)?;
        if !inner.next().is_none() {
            return Err(ParseError::UnexpectedInner);
        }

        return match expr_type.as_rule() {
            Rule::return_stmt => {
                let expr_inner = expr_type.into_inner();
                return if expr_inner.clone().count() == 1 {
                    Ok(StatementType::RETURN)
                } else if expr_inner.count() == 0 {
                    Ok(StatementType::RETURN)
                } else {
                    Err(ParseError::UnexpectedInner)
                };
            }
            Rule::assignment => Ok(StatementType::ASSIGNMENT(Assignment::try_from(expr_type)?)),
            Rule::reassignment => Ok(StatementType::REASSIGNMENT(Assignment::try_from(
                expr_type,
            )?)),
            Rule::fn_call => Ok(StatementType::FN_CALL(FnCall::try_from(expr_type)?)),
            Rule::if_stmt => Ok(StatementType::IF_STATEMENT(IfStmt::try_from(expr_type)?)),
            Rule::while_stmt => Ok(StatementType::WHILE_STATEMENT(WhileStmt::try_from(
                expr_type,
            )?)),
            Rule::do_while_stmt => Ok(StatementType::DO_WHILE_STATEMENT(DoWhileStmt::try_from(
                expr_type,
            )?)),
            _ => Err(ParseError::NoMatch(String::from(
                "StatementType::from_pair",
            ))),
        };
    }

    fn pre_visit(&self, visitor: &mut dyn AstVisitor) -> compile::Result<()> {
        match self {
            Self::RETURN => visitor.pre_visit_StatementType_RETURN(&false)?,
            Self::ASSIGNMENT(ass) => visitor.pre_visit_StatementType_ASSIGNMENT(ass)?,
            Self::REASSIGNMENT(reass) => visitor.pre_visit_StatementType_REASSIGNMENT(reass)?,
            Self::FN_CALL(fn_call) => visitor.pre_visit_StatementType_FN_CALL(fn_call)?,
            Self::IF_STATEMENT(if_stmt) => visitor.pre_visit_StatementType_IF_STATEMENT(if_stmt)?,
            Self::WHILE_STATEMENT(while_stmt) => {
                visitor.pre_visit_StatementType_WHILE_STATEMENT(while_stmt)?
            }
            Self::DO_WHILE_STATEMENT(do_while) => {
                visitor.pre_visit_StatementType_DO_WHILE_STATEMENT(do_while)?
            }
            _ => unimplemented!(),
        }
        Ok(())
    }

    fn visit(&self, visitor: &mut dyn AstVisitor) -> compile::Result<()> {
        match self {
            Self::RETURN => visitor.visit_StatementType_RETURN(&false)?,
            Self::ASSIGNMENT(ass) => visitor.visit_StatementType_ASSIGNMENT(ass)?,
            Self::REASSIGNMENT(reass) => visitor.visit_StatementType_REASSIGNMENT(reass)?,
            Self::FN_CALL(fn_call) => visitor.visit_StatementType_FN_CALL(fn_call)?,
            Self::IF_STATEMENT(if_stmt) => visitor.visit_StatementType_IF_STATEMENT(if_stmt)?,
            Self::WHILE_STATEMENT(while_stmt) => {
                visitor.visit_StatementType_WHILE_STATEMENT(while_stmt)?
            }
            Self::DO_WHILE_STATEMENT(do_while) => {
                visitor.visit_StatementType_DO_WHILE_STATEMENT(do_while)?
            }
            _ => unimplemented!(),
        }
        Ok(())
    }

    fn post_visit(&self, visitor: &mut dyn AstVisitor) -> compile::Result<()> {
        match self {
            Self::RETURN => visitor.post_visit_StatementType_RETURN(&false)?,
            Self::ASSIGNMENT(ass) => visitor.post_visit_StatementType_ASSIGNMENT(ass)?,
            Self::REASSIGNMENT(reass) => visitor.post_visit_StatementType_REASSIGNMENT(reass)?,
            Self::FN_CALL(fn_call) => visitor.post_visit_StatementType_FN_CALL(fn_call)?,
            Self::IF_STATEMENT(if_stmt) => {
                visitor.post_visit_StatementType_IF_STATEMENT(if_stmt)?
            }
            Self::WHILE_STATEMENT(while_stmt) => {
                visitor.post_visit_StatementType_WHILE_STATEMENT(while_stmt)?
            }
            Self::DO_WHILE_STATEMENT(do_while) => {
                visitor.post_visit_StatementType_DO_WHILE_STATEMENT(do_while)?
            }
            _ => unimplemented!(),
        }
        Ok(())
    }
}
impl StatementType {
    fn get_expression(&self, pair: Pair<Rule>) -> Result<Option<Expression>> {
        return match pair.as_rule() {
            Rule::statement => {
                let mut inner = pair.into_inner();
                let expr_type = inner.next().ok_or(ParseError::ExpectedInner)?;
                if !inner.next().is_none() {
                    return Err(ParseError::UnexpectedInner);
                }

                let mut expr_inner = expr_type.clone().into_inner();
                return match expr_type.as_rule() {
                    Rule::return_stmt => {
                        return if expr_inner.clone().count() == 1 {
                            Ok(Some(Expression::try_from(
                                expr_inner.next().ok_or(ParseError::ExpectedInner)?,
                            )?))
                        } else {
                            Ok(None)
                        }
                    }
                    Rule::assignment => {
                        // Assignment's first inner is an ident we need to skip over to find the real expression
                        expr_inner.next().ok_or(ParseError::ExpectedInner)?;
                        Ok(Some(Expression::try_from(
                            expr_inner.next().ok_or(ParseError::ExpectedInner)?,
                        )?))
                    }
                    Rule::reassignment => {
                        // Reassignment's first inner is an ident we need to skip over to find the real expression
                        expr_inner.next().ok_or(ParseError::ExpectedInner)?;
                        Ok(Some(Expression::try_from(
                            expr_inner.next().ok_or(ParseError::ExpectedInner)?,
                        )?))
                    }
                    Rule::fn_call => Ok(None),
                    Rule::if_stmt => Ok(None),
                    Rule::while_stmt => Ok(None),
                    Rule::do_while_stmt => Ok(None),
                    _ => unimplemented!(),
                };
            }
            _ => Err(ParseError::IncorrectRule(
                String::from(stringify!(Expression)),
                String::from(stringify!(Rule::statement)),
                format!("{:?}", pair.as_rule()),
            )),
        };
    }
}

try_from_pair!(IfStmt, Rule::if_stmt);
impl AstNode for IfStmt {
    fn from_pair(pair: Pair<Rule>) -> Result<Self>
    where
        Self: Sized,
    {
        let mut inner = pair.into_inner();
        let expr = Expression::try_from(inner.next().ok_or(ParseError::ExpectedInner)?)?;
        let then = CodeBlock::try_from(inner.next().ok_or(ParseError::ExpectedInner)?)?;
        let else_stmt = match inner.next() {
            None => None,
            Some(else_pair) => Some(Box::new(ElseStmt::try_from(else_pair)?)),
        };

        return Ok(Self {
            expr,
            then,
            else_stmt,
        });
    }

    visit_fns!(IfStmt);
}

try_from_pair!(WhileStmt, Rule::while_stmt);
impl AstNode for WhileStmt {
    fn from_pair(pair: Pair<Rule>) -> Result<Self>
    where
        Self: Sized,
    {
        let mut inner = pair.into_inner();
        let expr = Expression::try_from(inner.next().ok_or(ParseError::ExpectedInner)?)?;
        let then = CodeBlock::try_from(inner.next().ok_or(ParseError::ExpectedInner)?)?;

        return Ok(Self { expr, then });
    }

    visit_fns!(WhileStmt);
}

try_from_pair!(DoWhileStmt, Rule::do_while_stmt);
impl AstNode for DoWhileStmt {
    fn from_pair(pair: Pair<Rule>) -> Result<Self>
    where
        Self: Sized,
    {
        let mut inner = pair.into_inner();
        let then = CodeBlock::try_from(inner.next().ok_or(ParseError::ExpectedInner)?)?;
        let expr = Expression::try_from(inner.next().ok_or(ParseError::ExpectedInner)?)?;

        return Ok(Self { expr, then });
    }

    visit_fns!(DoWhileStmt);
}

try_from_pair!(ElseStmt, Rule::else_stmt);
impl AstNode for ElseStmt {
    fn from_pair(pair: Pair<Rule>) -> Result<Self>
    where
        Self: Sized,
    {
        let mut inner = pair.into_inner();
        let next = inner.next().ok_or(ParseError::ExpectedInner)?;
        return match next.clone().as_rule() {
            Rule::code_block => Ok(ElseStmt::ELSE(CodeBlock::try_from(next)?)),
            Rule::if_stmt => Ok(ElseStmt::ELIF(IfStmt::try_from(next)?)),
            _ => Err(ParseError::NoMatch(String::from("ElseStmt::from_pair"))),
        };
    }

    fn pre_visit(&self, visitor: &mut dyn AstVisitor) -> compile::Result<()> {
        match self {
            Self::ELSE(else_stmt) => visitor.pre_visit_ElseStmt_ELSE(else_stmt)?,
            Self::ELIF(elif) => visitor.pre_visit_ElseStmt_ELIF(elif)?,
        }
        Ok(())
    }

    fn visit(&self, visitor: &mut dyn AstVisitor) -> compile::Result<()> {
        match self {
            Self::ELSE(else_stmt) => visitor.visit_ElseStmt_ELSE(else_stmt)?,
            Self::ELIF(elif) => visitor.visit_ElseStmt_ELIF(elif)?,
        }
        Ok(())
    }

    fn post_visit(&self, visitor: &mut dyn AstVisitor) -> compile::Result<()> {
        match self {
            Self::ELSE(else_stmt) => visitor.post_visit_ElseStmt_ELSE(else_stmt)?,
            Self::ELIF(elif) => visitor.post_visit_ElseStmt_ELIF(elif)?,
        }
        Ok(())
    }
}

try_from_pair!(Expression, Rule::expression);
#[allow(unreachable_patterns)]
impl AstNode for Expression {
    fn from_pair(pair: Pair<Rule>) -> Result<Self>
    where
        Self: Sized,
    {
        let mut inner = pair.into_inner();
        let expr_type = inner.next().ok_or(ParseError::ExpectedInner)?;
        if !inner.next().is_none() {
            return Err(ParseError::UnexpectedInner);
        }

        return match expr_type.as_rule() {
            Rule::primitive => Ok(Expression::PRIM(Primitive::try_from(expr_type)?)),
            Rule::fn_call => Ok(Expression::FN_CALL(FnCall::try_from(expr_type)?)),
            Rule::ident => Ok(Expression::VARIABLE(Ident::from(expr_type.as_str()))),
            _ => Err(ParseError::NoMatch(String::from("Expression::from_pair"))),
        };
    }

    fn pre_visit(&self, visitor: &mut dyn AstVisitor) -> compile::Result<()> {
        match self {
            Self::PRIM(prim) => visitor.pre_visit_Expression_PRIM(prim)?,
            Self::FN_CALL(fn_call) => visitor.pre_visit_Expression_FN_CALL(fn_call)?,
            Self::VARIABLE(fn_call) => visitor.pre_visit_Expression_VARIABLE(fn_call)?,
            _ => unimplemented!(),
        }
        Ok(())
    }

    fn visit(&self, visitor: &mut dyn AstVisitor) -> compile::Result<()> {
        match self {
            Self::PRIM(prim) => visitor.visit_Expression_PRIM(prim)?,
            Self::FN_CALL(fn_call) => visitor.visit_Expression_FN_CALL(fn_call)?,
            Self::VARIABLE(fn_call) => visitor.visit_Expression_VARIABLE(fn_call)?,
            _ => unimplemented!(),
        }
        Ok(())
    }

    fn post_visit(&self, visitor: &mut dyn AstVisitor) -> compile::Result<()> {
        match self {
            Self::PRIM(prim) => visitor.post_visit_Expression_PRIM(prim)?,
            Self::FN_CALL(fn_call) => visitor.post_visit_Expression_FN_CALL(fn_call)?,
            Self::VARIABLE(fn_call) => visitor.post_visit_Expression_VARIABLE(fn_call)?,
            _ => unimplemented!(),
        }
        Ok(())
    }
}

try_from_pair!(Primitive, Rule::primitive);
#[allow(unreachable_patterns)]
impl AstNode for Primitive {
    fn from_pair(pair: Pair<Rule>) -> Result<Self>
    where
        Self: Sized,
    {
        let mut inner = pair.into_inner();
        let prim_type = inner.next().ok_or(ParseError::ExpectedInner)?;
        if !inner.next().is_none() {
            return Err(ParseError::UnexpectedInner);
        }

        return match prim_type.as_rule() {
            Rule::uint64_primitive => Ok(Primitive::UINT(prim_type.as_str().parse()?)),
            Rule::string_primitive => Ok(Primitive::STRING(parse_string(
                prim_type
                    .into_inner()
                    .next()
                    .ok_or(ParseError::ExpectedInner)?
                    .as_str(),
            ))),
            Rule::bool_primitive => Ok(Primitive::BOOL(prim_type.as_str() == "true")),
            _ => Err(ParseError::NoMatch(String::from("Primitive::from_pair"))),
        };
    }

    fn pre_visit(&self, visitor: &mut dyn AstVisitor) -> compile::Result<()> {
        match self {
            Self::UINT(value) => visitor.pre_visit_Primitive_UINT64(value)?,
            Self::STRING(value) => visitor.pre_visit_Primitive_STRING(value)?,
            Self::BOOL(value) => visitor.pre_visit_Primitive_BOOL(value)?,
            _ => unimplemented!(),
        }
        Ok(())
    }

    fn visit(&self, visitor: &mut dyn AstVisitor) -> compile::Result<()> {
        match self {
            Self::UINT(value) => visitor.visit_Primitive_UINT64(value)?,
            Self::STRING(value) => visitor.visit_Primitive_STRING(value)?,
            Self::BOOL(value) => visitor.visit_Primitive_BOOL(value)?,
            _ => unimplemented!(),
        }
        Ok(())
    }

    fn post_visit(&self, visitor: &mut dyn AstVisitor) -> compile::Result<()> {
        match self {
            Self::UINT(value) => visitor.post_visit_Primitive_UINT64(value)?,
            Self::STRING(value) => visitor.post_visit_Primitive_STRING(value)?,
            Self::BOOL(value) => visitor.post_visit_Primitive_BOOL(value)?,

            _ => unimplemented!(),
        }
        Ok(())
    }
}

try_from_pair!(CrabType, Rule::crab_type);
impl AstNode for CrabType {
    fn from_pair(pair: Pair<Rule>) -> Result<Self>
    where
        Self: Sized,
    {
        match pair.as_str() {
            "Int" => Ok(Self::UINT),
            "String" => Ok(Self::STRING),
            "Float" => Ok(Self::FLOAT),
            "Bool" => Ok(Self::BOOL),
            s => Err(ParseError::InvalidCrabType(String::from(s))),
        }
    }

    visit_fns!(CrabType);
}
impl<'ctx> CrabType {
    pub fn as_llvm_type(&self, context: &'ctx Context) -> AnyTypeEnum<'ctx> {
        return match self {
            Self::UINT => AnyTypeEnum::IntType(context.i64_type()),
            // TODO: Figure out what to do about address spaces
            Self::STRING => {
                AnyTypeEnum::PointerType(context.i8_type().ptr_type(AddressSpace::Generic))
            }
            Self::FLOAT => AnyTypeEnum::FloatType(context.f64_type()),
            Self::BOOL => AnyTypeEnum::IntType(context.custom_width_int_type(1)),
            Self::VOID => AnyTypeEnum::VoidType(context.void_type()),
        };
    }

    pub fn try_as_basic_type(
        &self,
        context: &'ctx Context,
    ) -> compile::Result<BasicTypeEnum<'ctx>> {
        return match self {
            Self::UINT => Ok(BasicTypeEnum::IntType(context.i64_type())),
            // TODO: Figure out what to do about address spaces
            Self::STRING => Ok(BasicTypeEnum::PointerType(
                context.i8_type().ptr_type(AddressSpace::Generic),
            )),
            Self::FLOAT => Ok(BasicTypeEnum::FloatType(context.f64_type())),
            Self::BOOL => Ok(BasicTypeEnum::IntType(context.custom_width_int_type(1))),
            Self::VOID => Err(CompileError::InvalidArgType(String::from(stringify!(
                CrabType::Void
            )))),
        };
    }

    pub fn try_as_basic_metadata_type(
        &self,
        context: &'ctx Context,
    ) -> compile::Result<BasicMetadataTypeEnum<'ctx>> {
        Ok(BasicMetadataTypeEnum::from(
            self.try_as_basic_type(context)?,
        ))
    }

    pub fn as_fn_type(
        &self,
        context: &'ctx Context,
        args: &[TypedIdent],
        variadic: bool,
    ) -> compile::Result<FunctionType<'ctx>> {
        trace!("CrabType as fn_type");

        let mut param_vec = vec![];
        for ti in args {
            param_vec.push(ti.crab_type.try_as_basic_metadata_type(context)?);
        }
        let param_types = param_vec.as_slice();

        return match self {
            Self::UINT => Ok(context.i64_type().fn_type(param_types, variadic)),
            // TODO: Figure out what to do about address spaces
            Self::STRING => Ok(context
                .i8_type()
                .ptr_type(AddressSpace::Generic)
                .fn_type(param_types, false)),
            Self::BOOL => Ok(context
                .custom_width_int_type(1)
                .fn_type(param_types, variadic)),
            Self::FLOAT => Ok(context.f64_type().fn_type(param_types, variadic)),
            Self::VOID => Ok(context.void_type().fn_type(param_types, variadic)),
        };
    }
}

try_from_pair!(FnCall, Rule::fn_call);
impl AstNode for FnCall {
    fn from_pair(pair: Pair<Rule>) -> Result<Self>
    where
        Self: Sized,
    {
        let mut inner = pair.into_inner();
        let name = Ident::from(inner.next().ok_or(ParseError::ExpectedInner)?.as_str());
        let args = match inner.next() {
            None => ExpressionList {
                expressions: vec![],
            },
            Some(n) => ExpressionList::try_from(n)?,
        };
        Ok(Self { name, args })
    }

    visit_fns!(FnCall);
}

try_from_pair!(TypedIdent, Rule::typed_ident);
impl AstNode for TypedIdent {
    fn from_pair(pair: Pair<Rule>) -> Result<Self>
    where
        Self: Sized,
    {
        let mut inner = pair.into_inner();
        let crab_type = CrabType::try_from(inner.next().ok_or(ParseError::ExpectedInner)?)?;
        let name = Ident::from(inner.next().ok_or(ParseError::ExpectedInner)?.as_str());

        Ok(Self { name, crab_type })
    }

    visit_fns!(TypedIdent);
}

try_from_pair!(IdentList, Rule::ident_list);
impl AstNode for IdentList {
    fn from_pair(pair: Pair<Rule>) -> Result<Self>
    where
        Self: Sized,
    {
        let mut idents = vec![];

        for ident in pair.into_inner() {
            idents.push(Ident::from(ident.as_str()));
        }

        Ok(Self { idents })
    }

    visit_fns!(IdentList);
}

try_from_pair!(TypedIdentList, Rule::typed_ident_list);
impl AstNode for TypedIdentList {
    fn from_pair(pair: Pair<Rule>) -> Result<Self>
    where
        Self: Sized,
    {
        let mut typed_idents = vec![];

        for ty_id in pair.into_inner() {
            typed_idents.push(TypedIdent::try_from(ty_id)?);
        }

        Ok(Self { typed_idents })
    }

    visit_fns!(TypedIdentList);
}

try_from_pair!(ExpressionList, Rule::expression_list);
impl AstNode for ExpressionList {
    fn from_pair(pair: Pair<Rule>) -> Result<Self>
    where
        Self: Sized,
    {
        let mut expressions = vec![];

        for expr in pair.into_inner() {
            expressions.push(Expression::try_from(expr)?);
        }

        Ok(Self { expressions })
    }

    visit_fns!(ExpressionList);
}

/// Assignment requries a custom TryFrom implementation because it can be built from two different rules
impl TryFrom<Pair<'_, Rule>> for Assignment {
    type Error = ParseError;
    fn try_from(pair: Pair<Rule>) -> std::result::Result<Assignment, Self::Error> {
        match pair.as_rule() {
            Rule::assignment => Assignment::from_pair(pair),
            Rule::reassignment => Assignment::from_pair(pair),
            _ => Err(ParseError::IncorrectRule(
                String::from(stringify!(Assignment)),
                format!(
                    "{} or {}",
                    stringify!(Rule::assignment),
                    stringify!(Rule::reassignment)
                ),
                format!("{:?}", pair.as_rule()),
            )),
        }
    }
}
impl AstNode for Assignment {
    fn from_pair(pair: Pair<Rule>) -> Result<Self> {
        let mut inner = pair.into_inner();
        let var_name = Ident::from(inner.next().ok_or(ParseError::ExpectedInner)?.as_str());

        Ok(Self { var_name })
    }

    visit_fns!(Assignment);
}
