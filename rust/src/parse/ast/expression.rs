use crate::parse::ast::{AstNode, FnCall, Ident, Operator, Primitive, StructFieldInit, StructInit};
use crate::parse::ParseError::ExpectedInner;
use crate::parse::{ParseError, Result, Rule};
use crate::try_from_pair;
use crate::util::{int_struct_name, new_string_name, primitive_field_name};
use pest::iterators::Pair;
use std::convert::TryFrom;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Expression {
    pub this: ExpressionType,
    pub next: Option<Box<Expression>>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[allow(non_camel_case_types)]
pub enum ExpressionType {
    PRIM(Primitive),
    STRUCT_INIT(StructInit),
    FN_CALL(FnCall),
    VARIABLE(Ident),
}

try_from_pair!(Expression, Rule::expression);
#[allow(unreachable_patterns)]
impl AstNode for Expression {
    fn from_pair(pair: Pair<Rule>) -> Result<Self>
    where
        Self: Sized,
    {
        let mut inner = pair.into_inner();
        let first_pair = inner.next().ok_or(ParseError::ExpectedInner)?;

        let mut expr = Expression {
            this: ExpressionType::try_from(first_pair)?,
            next: None,
        };

        // This will continue appending tokens to the expression until either:
        // A) We run out of tokens
        // B) We encounter an operator
        // If we do encounter an operator, we will have consumed it :(
        // Therefore, we have to clone the iterator
        for pair in inner.clone() {
            match ExpressionType::try_from(pair) {
                Ok(et) => expr.append(et),
                Err(_) => break,
            }
        }

        // Skip all of the elements we added to the expression
        // Subtract 1 because the first element was consumed from the iterator before we cloned it
        let mut inner = inner.skip(expr.get_depth() - 1);

        // If there is an operator, add the appropriate function call to the end of the expression chain
        //TODO: Order of operations
        if let Some(operator_pair) = inner.next() {
            let operator = Operator::try_from(operator_pair)?;
            let arg = Expression::try_from(inner.next().ok_or(ExpectedInner)?)?;

            expr.append(ExpressionType::FN_CALL(FnCall {
                name: operator.into_fn_name(),
                unnamed_args: vec![arg],
                named_args: vec![],
            }));
        }

        Ok(expr)
    }
}
impl Expression {
    ///
    /// Adds an ExpressionType to the end of this Expression
    /// This will recursively travel down the tree until it hits the end
    ///
    /// Params:
    /// - `addition`: The ExpressionType to add to this expression
    ///
    fn append(&mut self, addition: ExpressionType) {
        match &mut self.next {
            None => {
                self.next = Some(Box::new(Expression {
                    this: addition,
                    next: None,
                }))
            }
            Some(expr) => expr.append(addition),
        }
    }

    ///
    /// Get the number of tokens this expression has
    /// Recursively travels the expression tree and counts the number of ExpressionTypes it found
    ///
    fn get_depth(&self) -> usize {
        match &self.next {
            None => 1,
            Some(n) => n.get_depth() + 1,
        }
    }
}

impl TryFrom<Pair<'_, Rule>> for ExpressionType {
    type Error = ParseError;

    fn try_from(pair: Pair<'_, Rule>) -> std::result::Result<Self, Self::Error> {
        match pair.clone().as_rule() {
            // Primitives are *special*. They need to be converted to StructInits that contain a Primitive argument
            Rule::primitive => {
                let prim = Primitive::try_from(pair)?;
                match &prim {
                    Primitive::UINT(_) => Ok(Self::STRUCT_INIT(StructInit {
                        name: int_struct_name(),
                        fields: vec![StructFieldInit {
                            name: primitive_field_name(),
                            value: Expression {
                                this: ExpressionType::PRIM(prim),
                                next: None,
                            },
                        }],
                    })),
                    Primitive::STRING(str) => Ok(Self::FN_CALL(FnCall {
                        name: new_string_name(),
                        unnamed_args: vec![
                            Expression {
                                this: ExpressionType::PRIM(prim.clone()),
                                next: None,
                            },
                            Expression {
                                this: ExpressionType::PRIM(Primitive::UINT((str.len() + 1) as u64)),
                                next: None,
                            },
                        ],
                        named_args: vec![],
                    })),
                    _ => Ok(Self::PRIM(prim)),
                }
            }
            Rule::struct_init => Ok(Self::STRUCT_INIT(StructInit::try_from(pair)?)),
            Rule::fn_call => Ok(Self::FN_CALL(FnCall::try_from(pair)?)),
            Rule::ident => Ok(Self::VARIABLE(Ident::from(pair.as_str()))),
            _ => Err(ParseError::NoMatch(String::from(
                "ExpressionType::try_from<Pair>",
            ))),
        }
    }
}
