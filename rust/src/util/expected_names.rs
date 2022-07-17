use crate::parse::ast::{Ident, StructIdent};

pub fn int_struct_id() -> StructIdent {
    StructIdent {
        name: Ident::from("Int"),
        tmpls: vec![],
    }
}
pub fn string_type_id() -> StructIdent {
    StructIdent {
        name: Ident::from("String"),
        tmpls: vec![],
    }
}
pub fn bool_struct_id() -> StructIdent {
    StructIdent {
        name: Ident::from("Bool"),
        tmpls: vec![],
    }
}
pub fn list_struct_id() -> StructIdent {
    StructIdent {
        name: Ident::from("List"),
        tmpls: vec![],
    }
}

pub fn tmpl_param_name() -> Ident { Ident::from("T") }

pub fn primitive_field_name() -> Ident {
    Ident::from("value")
}
pub fn capacity_field_name() -> Ident { Ident::from("capacity") }
pub fn length_field_name() -> Ident { Ident::from("length") }

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
