use crate::compile::Result;
use crate::parse::{
    Assignment, AstNode, CodeBlock, CrabAst, CrabType, DoWhileStmt, FnCall, FnParam, Func,
    FuncSignature, Ident, IfStmt, NamedExpression, NamedFnParam, Primitive, Statement, WhileStmt,
};

macro_rules! second_dispatch_fns {
    ($node_type:ident) => {
        paste::item! {
            fn [< pre_visit_ $node_type >](&mut self, _node: &$node_type) -> Result<()> {
                // do nothing
                Ok(())
            }
            fn [< visit_ $node_type >](&mut self, _node: &$node_type) -> Result<()> {
                // do nothing
                Ok(())
            }
            fn [< post_visit_ $node_type >](&mut self, _node: &$node_type) -> Result<()> {
                // do nothing
                Ok(())
            }
        }
    };

    ($node_type:ident, $($nodes_type:ident),+) => {
        second_dispatch_fns!($node_type);
        second_dispatch_fns!($($nodes_type),+);
    };
}

macro_rules! second_dispatch_enums {
    (($enum_type:ident, $enum_value:ident, $enum_inner:ty)) => {
        paste::item! {
            fn [< pre_visit_ $enum_type _ $enum_value >](&mut self, _node: &$enum_inner) -> Result<()> {
                // do nothing
                Ok(())
            }
            fn [< visit_ $enum_type _ $enum_value >](&mut self, _node: &$enum_inner) -> Result<()> {
                // do nothing
                Ok(())
            }
            fn [< post_visit_ $enum_type _ $enum_value >](&mut self, _node: &$enum_inner) -> Result<()> {
                // do nothing
                Ok(())
            }
        }
    };

    (($enum_type:ident, $enum_value:ident, $enum_inner:ty), $(($enums_type:ident, $enums_value:ident, $enums_inner:ty)),+) => {
        second_dispatch_enums!(($enum_type, $enum_value, $enum_inner));
        second_dispatch_enums!($(($enums_type, $enums_value, $enums_inner)),+);
    };
}

#[allow(non_snake_case)]
pub trait AstVisitor {
    fn pre_visit(&mut self, node: &dyn AstNode) -> Result<()>;
    fn visit(&mut self, node: &dyn AstNode) -> Result<()>;

    second_dispatch_fns! {
        CrabAst,
        Func,
        FuncSignature,
        CodeBlock,
        Ident,
        CrabType,
        FnCall,
        Assignment,
        Statement,
        //TypedIdent,
        //IdentList,
        //TypedIdentList,
        //ExpressionList,
        IfStmt,
        WhileStmt,
        DoWhileStmt,
        FnParam,
        NamedFnParam,
        NamedExpression
    }

    // The third argument for StatementType::RETURN is a throwaway and not actually used
    second_dispatch_enums! {
        (Primitive, UINT64, u64),
        (Primitive, STRING, String),
        (Primitive, BOOL, bool),
        (Expression, PRIM, Primitive),
        (Expression, FN_CALL, FnCall),
        (Expression, VARIABLE, Ident),
        (StatementType, RETURN, bool),
        (StatementType, ASSIGNMENT, Assignment),
        (StatementType, REASSIGNMENT, Assignment),
        (StatementType, FN_CALL, FnCall),
        (StatementType, IF_STATEMENT, IfStmt),
        (StatementType, WHILE_STATEMENT, WhileStmt),
        (StatementType, DO_WHILE_STATEMENT, DoWhileStmt),
        (ElseStmt, ELSE, CodeBlock),
        (ElseStmt, ELIF, IfStmt)
    }
}
