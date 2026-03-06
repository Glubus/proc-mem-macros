use super::types::type_to_read_fn;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Result, Type};

/// Build `addr_ts.read_T(offset_ts)?`, with a `FieldType::from(…)` wrapper when `via` is set.
///
/// # Errors
///
/// Returns an error if neither `field_ty` nor `via` maps to a known `read_*`
/// method, with a message pointing the user toward `via` or `#[nested]`.
pub fn individual_read_expr(
    field_ty: &Type,
    via: &Option<Type>,
    addr_ts: &TokenStream,
    offset_ts: &TokenStream,
) -> Result<TokenStream> {
    let effective = via.as_ref().unwrap_or(field_ty);

    if let Some(read_fn) = type_to_read_fn(effective) {
        let ts = if via.is_some() {
            quote! { #field_ty::from(#addr_ts.#read_fn(#offset_ts)?) }
        } else {
            quote! { #addr_ts.#read_fn(#offset_ts)? }
        };
        Ok(ts)
    } else {
        Err(syn::Error::new_spanned(
            field_ty,
            "cannot infer read function — use `via = T` for From<T> conversions, \
             or `#[nested(offset)]` for types that implement read_from_memory",
        ))
    }
}
