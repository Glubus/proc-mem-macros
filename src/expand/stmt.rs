use crate::attr::FieldAttr;
use crate::config::StructConfig;
use crate::utils::individual_read_expr;
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{Result, Type};

/// Emit a `let name: T = …;` statement for one non-batchable field.
///
/// `PtrChain` emits a scoped block instead of a plain `let` because it needs
/// a mutable local (`__pc_addr`) to walk intermediate pointer dereferences.
pub fn gen_field_stmt(
    name: &Ident,
    ty: &Type,
    fa: Option<FieldAttr>,
    config: &StructConfig,
) -> Result<TokenStream> {
    match fa {
        Some(FieldAttr::Offset { expr, via }) => {
            let rexpr =
                individual_read_expr(ty, &via, &quote! { p }, &quote! { __base + (#expr) })?;
            Ok(quote! { let #name: #ty = #rexpr; })
        }

        Some(FieldAttr::Nested(expr)) => {
            if config.state.is_some() {
                Ok(quote! {
                    let #name: #ty = #ty::read_from_memory(p, state, __base + (#expr))?;
                })
            } else {
                Ok(quote! {
                    let #name: #ty = #ty::read_from_memory(p, __base + (#expr))?;
                })
            }
        }

        Some(FieldAttr::PtrChain { base, offsets, via }) => {
            let ptr_fn = &config.ptr_fn;
            let addr_ty = &config.addr;
            let (derefs, tail) = offsets.split_at(offsets.len() - 1);
            let deref_steps = derefs
                .iter()
                .map(|off| quote! { __pc_addr = p.#ptr_fn(__pc_addr + (#off))?; });
            let foff = &tail[0];
            let rexpr =
                individual_read_expr(ty, &via, &quote! { p }, &quote! { __pc_addr + (#foff) })?;
            Ok(quote! {
                let #name: #ty = {
                    let mut __pc_addr: #addr_ty = #base;
                    #( #deref_steps )*
                    #rexpr
                };
            })
        }

        Some(FieldAttr::Computed(expr)) => Ok(quote! { let #name: #ty = #expr; }),

        Some(FieldAttr::Skip) => Ok(quote! { let #name: #ty = Default::default(); }),

        None => Err(syn::Error::new_spanned(
            name,
            "field needs a ReadMemory attribute \
             (#[offset(...)], #[nested(...)], #[ptr_chain(...)], #[computed(...)], or #[skip])",
        )),
    }
}
