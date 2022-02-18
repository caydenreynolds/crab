use crate::parse::ast::Ident;

pub fn mangle_function_name(fn_name: &Ident, caller_name: Option<&Ident>) -> Ident {
    match caller_name {
        None => format!("_FN_{}", fn_name),
        Some(sn) => format!("_MD_{}_{}", sn, fn_name),
    }
}
