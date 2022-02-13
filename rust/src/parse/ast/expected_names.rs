use crate::parse::ast::Ident;

pub fn int_struct_name() -> Ident { Ident::from("Int") }
pub fn primitive_field_name() -> Ident { Ident::from("value") }
pub fn main_func_name() -> Ident { Ident::from("main") }
pub fn internal_main_func_name() -> Ident { Ident::from("__main__") }
