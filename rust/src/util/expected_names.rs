use crate::parse::ast::Ident;

pub fn int_struct_name() -> Ident {
    Ident::from("Int")
}
pub fn string_struct_name() -> Ident {
    Ident::from("String")
}
pub fn bool_struct_name() -> Ident {
    Ident::from("Bool")
}
pub fn list_struct_name() -> Ident {
    Ident::from("List")
}
pub fn primitive_field_name() -> Ident {
    Ident::from("value")
}
pub fn length_field_name() -> Ident {
    Ident::from("length")
}
pub fn capacity_field_name() -> Ident {
    Ident::from("capacity")
}
pub fn main_func_name() -> Ident {
    Ident::from("main")
}
pub fn magic_main_func_name() -> Ident {
    Ident::from("__main__")
}
pub fn printf_c_name() -> Ident {
    Ident::from("__c_print_str__")
}
pub fn strlen_c_name() -> Ident {
    Ident::from("__c_strlen__")
}
pub fn printf_crab_name() -> Ident {
    Ident::from("__printf__")
}
pub fn format_i_c_name() -> Ident {
    Ident::from("__c_format_i__")
}
pub fn new_list_name() -> Ident {
    Ident::from("__new_list__")
}
pub fn get_fn_name() -> Ident {
    Ident::from("get")
}
pub fn length_fn_name() -> Ident {
    Ident::from("len")
}
pub fn inner_add_fn_name() -> Ident {
    Ident::from("__inner_add__")
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
