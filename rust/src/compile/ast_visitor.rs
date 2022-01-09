use crate::parse::{
    AstNode, CodeBlock, CrabAst, Expression, Func, FuncSignature, Ident, Primitive,
};

macro_rules! second_dispatch_fns {
    ($node_type:ident) => {
        paste::item! {
            fn [< visit_ $node_type >](&mut self, _node: &$node_type) {
                // do nothing
            }
            fn [< post_visit_ $node_type >](&mut self, _node: &$node_type) {
                // do nothing
            }
        }
    };

    ($node_type:ident, $($nodes_type:ident),+) => {
        second_dispatch_fns!($node_type);
        second_dispatch_fns!($($nodes_type),+);
    };
}

macro_rules! second_dispatch_enums {
    ($enum_type:ident, $enum_value:ident, $enum_inner:ty) => {
        paste::item! {
            fn [< visit_ $enum_type _ $enum_value >](&mut self, _node: &$enum_inner) {
                // do nothing
            }
            fn [< post_visit_ $enum_type _ $enum_value >](&mut self, _node: &$enum_inner) {
                // do nothing
            }
        }
    };

    ($enum_type:ident, $enum_value:ident, $enum_inner:ty, $($enums_type:ident, $enums_value:ident, $enums_inner:ty),+) => {
        second_dispatch_enums!($enum_type, $enum_value, $enum_inner);
        second_dispatch_enums!($($enums_type, $enums_value, $enums_inner),+);
    };
}

#[allow(non_snake_case)]
pub trait AstVisitor {
    fn visit(&mut self, node: &dyn AstNode);

    second_dispatch_fns!(
        CrabAst,
        Func,
        FuncSignature,
        CodeBlock,
        Ident
    );

    second_dispatch_enums!(
        Primitive, UINT64, u64,
        Expression, PRIM, Primitive,
        Statement, RETURN, Option<Expression>
    );
}
