use crate::parse::ast::{PosParam, Ident};

///
/// Mangles a function name based on whether it's a function or a method
/// This is the first step towards completely mangling a function's name
///
/// Params:
/// * `fn_name` - The name of the function to mangle
/// * `caller_name` - The name of the Struct this function belongs to, if any
///
/// Returns:
/// The mangled function name
///
pub fn mangle_function_name(fn_name: &Ident, caller_name: Option<&Ident>) -> Ident {
    match caller_name {
        None => format!("_FN_{}", fn_name),
        Some(sn) => format!("_MD_{}_{}", sn, fn_name),
    }
}

///
/// The second (and final) step for fully mangling a function's name
/// Adds all of the names and types of the params_to_mangle to the mangled name
/// Only params that have an interface type need to be added here
///
/// Params:
/// * `mangled_name` - The name of the function already run through mangle_function_name
/// * `params_to_mangle` - All of the function params to add to the mangled name
///
/// Returns:
/// The fully mangled name
///
pub fn add_param_mangles(mangled_name: &Ident, params_to_mangle: &[PosParam]) -> Ident {
    let mut result = mangled_name.clone();
    for param in params_to_mangle {
        result = add_param_mangle(result, param);
    }
    result
}

///
/// Adds a single param to the mangled name
///
fn add_param_mangle(mangled_name: Ident, param_to_mangle: &PosParam) -> Ident {
    return format!(
        "{}_{}_{}",
        mangled_name, param_to_mangle.name, param_to_mangle.crab_type
    );
}
