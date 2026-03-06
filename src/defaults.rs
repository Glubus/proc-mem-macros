use proc_macro2::TokenStream;
use quote::quote;
use syn::{punctuated::Punctuated, Result};

use crate::attr::struct_attr::ReadMemoryStructAttr;

/// Walk the module, inject `#[read_memory_defaults(...)]` on every struct
/// that derives `ReadMemory`.
///
/// Injection works by synthesising a new attribute and pushing it onto the
/// struct's attribute list.  The derive macro picks it up later via
/// [`crate::expand::parse_struct_attrs`] and feeds it to
/// [`crate::config::StructConfig::resolve`] as the defaults layer.  This
/// avoids any global state: the defaults travel with the struct as a plain
/// attribute, visible to the same compiler pass that processes `ReadMemory`.
pub fn proc_mem_defaults_impl(attr: TokenStream, item: TokenStream) -> Result<TokenStream> {
    let defaults: ReadMemoryStructAttr = syn::parse2(attr)?;
    let mut module: syn::ItemMod = syn::parse2(item)?;

    if let Some((_, items)) = &mut module.content {
        for item in items.iter_mut() {
            if let syn::Item::Struct(s) = item {
                if has_derive_read_memory(s) {
                    inject_defaults_attr(s, &defaults)?;
                }
            }
        }
    }

    Ok(quote! { #module })
}

fn has_derive_read_memory(s: &syn::ItemStruct) -> bool {
    s.attrs.iter().any(|attr| {
        attr.path().is_ident("derive")
            && attr
                .parse_args_with(Punctuated::<syn::Path, syn::Token![,]>::parse_terminated)
                .map(|list| list.iter().any(|p| p.is_ident("ReadMemory")))
                .unwrap_or(false)
    })
}

fn inject_defaults_attr(s: &mut syn::ItemStruct, defaults: &ReadMemoryStructAttr) -> Result<()> {
    let mut parts: Vec<TokenStream> = vec![];

    if let Some(r) = &defaults.reader {
        parts.push(quote! { reader = #r });
    }
    if let Some(st) = &defaults.state {
        parts.push(quote! { state = #st });
    }
    if let Some(e) = &defaults.error {
        parts.push(quote! { error = #e });
    }
    if let Some(a) = &defaults.addr {
        parts.push(quote! { addr = #a });
    }
    if let Some(p) = &defaults.ptr_fn {
        parts.push(quote! { ptr_fn = #p });
    }

    if !parts.is_empty() {
        let new_attr: syn::Attribute = syn::parse_quote! {
            #[read_memory_defaults(#(#parts),*)]
        };
        s.attrs.push(new_attr);
    }

    Ok(())
}
