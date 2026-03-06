use crate::config::StructConfig;
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{Data, Fields, LitStr, Result, Type};

/// Parse the guard string into tokens and emit it verbatim.
///
/// The guard is a string literal in the attribute (e.g. `guard = "state.check()?"`).
/// It's a string — not a raw expr — because proc-macro attributes can't embed
/// arbitrary statements; the string is re-parsed as tokens at expansion time.
pub fn gen_guard_block(guard: &Option<LitStr>) -> Result<TokenStream> {
    let Some(guard_lit) = guard else {
        return Ok(quote! {});
    };

    let code: TokenStream = guard_lit.value().parse().map_err(|e| {
        syn::Error::new(guard_lit.span(), format!("invalid guard expression: {e}"))
    })?;

    Ok(quote! { #code })
}

/// Emit `let mut __base = base;` followed by one pointer-deref per chain step.
///
/// `__base` is `mut` because each chain step reassigns it:
/// `__base = p.read_i32(__base + step)?`.
pub fn gen_base_setup(config: &StructConfig) -> TokenStream {
    let addr_ty = &config.addr;
    let ptr_fn = &config.ptr_fn;
    let chain_steps = config
        .chain
        .iter()
        .map(|step| quote! { __base = p.#ptr_fn(__base + (#step))?; });
    quote! {
        let mut __base: #addr_ty = base;
        #( #chain_steps )*
    }
}

/// Emit the `read_from_memory` signature (no body).
///
/// Uses `::core::result::Result` to avoid clashing with a `Result` type alias
/// that the caller's crate might have in scope.
pub fn gen_fn_sig(config: &StructConfig) -> TokenStream {
    let reader = &config.reader;
    let error = &config.error;
    let addr = &config.addr;
    let state_param = opt_state_param(&config.state);
    quote! {
        pub fn read_from_memory(
            p: &'_ #reader,
            #state_param
            base: #addr,
        ) -> ::core::result::Result<Self, #error>
    }
}

/// Emit the `read(p [, state])` shorthand, or nothing if `base` was not set.
///
/// When `base = EXPR` is present the user almost always wants to call it
/// without computing the address themselves, hence the generated wrapper that
/// evaluates `EXPR` and forwards to `read_from_memory`.
pub fn gen_read_method(config: &StructConfig) -> TokenStream {
    let Some(init_base_expr) = &config.init_base else {
        return quote! {};
    };

    let reader = &config.reader;
    let error = &config.error;
    let state_param = opt_state_param(&config.state);
    let state_arg = opt_state_arg(&config.state);
    quote! {
        pub fn read(
            p: &'_ #reader,
            #state_param
        ) -> ::core::result::Result<Self, #error> {
            let base = #init_base_expr;
            Self::read_from_memory(p, #state_arg base)
        }
    }
}

pub fn extract_named_fields<'a>(
    data: &'a Data,
    name: &Ident,
) -> Result<impl Iterator<Item = &'a syn::Field>> {
    match data {
        Data::Struct(ds) => match &ds.fields {
            Fields::Named(f) => Ok(f.named.iter()),
            _ => Err(syn::Error::new_spanned(
                name,
                "ReadMemory only supports structs with named fields",
            )),
        },
        _ => Err(syn::Error::new_spanned(
            name,
            "ReadMemory can only be derived for structs",
        )),
    }
}

/// `state: &mut State,` when state is present, empty otherwise.
fn opt_state_param(state: &Option<Type>) -> TokenStream {
    match state {
        Some(s) => quote! { state: &mut #s, },
        None => quote! {},
    }
}

/// `state,` when state is present, empty otherwise (for call sites).
fn opt_state_arg(state: &Option<Type>) -> TokenStream {
    match state {
        Some(_) => quote! { state, },
        None => quote! {},
    }
}
