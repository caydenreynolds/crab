use crate::parse::ast::Ident;

pub fn int_struct_name() -> Ident {
    Ident::from("Int")
}
pub fn string_type_name() -> Ident {
    Ident::from("String")
}
pub fn bool_struct_name() -> Ident {
    Ident::from("Bool")
}
pub fn primitive_field_name() -> Ident {
    Ident::from("value")
}
pub fn main_func_name() -> Ident {
    Ident::from("main")
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

pub fn operator_add_name() -> Ident {
    Ident::from("operatorAdd")
}
pub fn operator_sub_name() -> Ident {
    Ident::from("operatorSub")
}
pub fn operator_mult_name() -> Ident {
    Ident::from("operatorMult")
}
pub fn operator_div_name() -> Ident {
    Ident::from("operatorDiv")
}
pub fn operator_eq_name() -> Ident {
    Ident::from("operatorEq")
}
pub fn operator_lt_name() -> Ident {
    Ident::from("operatorLt")
}
pub fn operator_gt_name() -> Ident {
    Ident::from("operatorGt")
}
pub fn operator_lte_name() -> Ident {
    Ident::from("operatorLte")
}
pub fn operator_gte_name() -> Ident {
    Ident::from("operatorGte")
}
pub fn operator_lsh_name() -> Ident {
    Ident::from("operatorLsh")
}
pub fn operator_rsh_name() -> Ident {
    Ident::from("operatorRsh")
}

pub fn to_string_name() -> Ident {
    Ident::from("toString")
}
