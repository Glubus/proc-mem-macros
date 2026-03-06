//! Code generation for `#[derive(ReadMemory)]`.
//!
//! Emits one `impl` block per struct containing `read_from_memory` and,
//! optionally, a `read` convenience wrapper.

pub mod batch;
pub mod fields;
pub mod stmt;
pub mod struct_setup;

use crate::attr::ReadMemoryStructAttr;
use crate::config::StructConfig;
use fields::process_fields;
use proc_macro2::TokenStream;
use quote::quote;
use struct_setup::{
    extract_named_fields, gen_base_setup, gen_fn_sig, gen_guard_block, gen_read_method,
};
use syn::{DeriveInput, Result};

pub fn derive_read_memory(input: DeriveInput) -> Result<TokenStream> {
    let struct_name = &input.ident;
    let (per_struct_attr, defaults_attr) = parse_struct_attrs(&input.attrs)?;
    let config = StructConfig::resolve(per_struct_attr, defaults_attr, struct_name.span())?;

    let guard_block = gen_guard_block(&config.guard)?;
    let base_setup = gen_base_setup(&config);
    let fn_sig = gen_fn_sig(&config);
    let read_method = gen_read_method(&config);

    let fields_iter = extract_named_fields(&input.data, struct_name)?;
    let (read_stmts, ctor_fields) = process_fields(fields_iter, &config)?;

    Ok(quote! {
        impl #struct_name {
            #fn_sig {
                #guard_block
                #base_setup
                #( #read_stmts )*

                Ok(Self {
                    #( #ctor_fields )*
                })
            }

            #read_method
        }
    })
}

/// Extract both attribute layers from a struct's attribute list.
///
/// `read_memory_defaults` is injected by [`crate::defaults`] — it isn't
/// written by the user, so both attributes default to all-`None` if absent.
fn parse_struct_attrs(
    attrs: &[syn::Attribute],
) -> Result<(ReadMemoryStructAttr, ReadMemoryStructAttr)> {
    let mut per_struct = ReadMemoryStructAttr::default();
    let mut defaults = ReadMemoryStructAttr::default();

    for attr in attrs {
        if attr.path().is_ident("read_memory") {
            per_struct = attr.parse_args::<ReadMemoryStructAttr>()?;
        } else if attr.path().is_ident("read_memory_defaults") {
            defaults = attr.parse_args::<ReadMemoryStructAttr>()?;
        }
    }

    Ok((per_struct, defaults))
}
