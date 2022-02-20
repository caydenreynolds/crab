use crate::parse::ast::Ident;

pub fn int_struct_name() -> Ident {
    Ident::from("Int")
}
pub fn string_type_name() -> Ident {
    Ident::from("String")
}
pub fn primitive_field_name() -> Ident {
    Ident::from("value")
}
pub fn main_func_name() -> Ident {
    Ident::from("main")
}
pub fn new_string_name() -> Ident {
    Ident::from("__new_string__")
}
