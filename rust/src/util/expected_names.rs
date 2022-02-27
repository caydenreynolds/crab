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
pub fn printf_c_name() -> Ident {
    Ident::from("printf")
}
pub fn printf_crab_name() -> Ident {
    Ident::from("__printf__")
}
pub fn format_i_c_name() -> Ident {
    Ident::from("__c_format_i__")
}
pub fn format_i_name() -> Ident {
    Ident::from("__format_i__")
}
pub fn add_int_name() -> Ident {
    Ident::from("__add_int__")
}
