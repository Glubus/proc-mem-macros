//! Field iteration with batch-run management.
//!
//! Batchable fields (consecutive `#[offset(LITERAL)]` with primitive types)
//! accumulate in a run and are flushed as one `p.read(base, len, &mut buf)?`
//! block.  Any non-batchable field breaks the current run.

use crate::attr::{extract_field_attr, FieldAttr};
use crate::config::StructConfig;
use crate::expand::batch::{gen_batch_block, BatchCandidate};
use crate::expand::stmt::gen_field_stmt;
use crate::utils::{expr_as_u64, primitive_byte_size, type_to_read_fn};
use proc_macro2::{Ident, Literal, TokenStream};
use quote::quote;
use syn::{Field, Result, Type};

/// Iterate `fields` and return `(read_stmts, ctor_fields)` in declaration order.
pub fn process_fields<'a>(
    fields: impl Iterator<Item = &'a Field>,
    config: &StructConfig,
) -> Result<(Vec<TokenStream>, Vec<TokenStream>)> {
    let mut read_stmts = vec![];
    let mut ctor_fields = vec![];
    let mut batch_run: Vec<BatchCandidate> = vec![];

    for field in fields {
        let field_name = field.ident.as_ref().expect("named field");
        let fa = extract_field_attr(&field.attrs)?;

        if let Some(candidate) = get_batch_candidate(field_name, &field.ty, &fa) {
            batch_run.push(candidate);
            ctor_fields.push(quote! { #field_name, });
            continue;
        }

        flush_batch(&mut batch_run, &mut read_stmts);
        let stmt = gen_field_stmt(field_name, &field.ty, fa, config)?;
        read_stmts.push(stmt);
        ctor_fields.push(quote! { #field_name, });
    }
    flush_batch(&mut batch_run, &mut read_stmts);
    Ok((read_stmts, ctor_fields))
}

/// Return `Some(candidate)` if the field can join a batch, `None` otherwise.
///
/// Batching requires `#[offset(LITERAL)]` (not a runtime expr) and a type
/// with a statically-known byte size.
fn get_batch_candidate(
    name: &Ident,
    field_ty: &Type,
    fa: &Option<FieldAttr>,
) -> Option<BatchCandidate> {
    match fa {
        Some(FieldAttr::Offset { expr, via }) => {
            let effective_ty = via.as_ref().unwrap_or(field_ty);
            let off = expr_as_u64(expr)?;
            let sz = primitive_byte_size(effective_ty)?;
            Some(BatchCandidate {
                name: name.clone(),
                field_ty: field_ty.clone(),
                raw_ty: effective_ty.clone(),
                offset: off,
                size: sz,
                needs_from: via.is_some(),
            })
        }
        _ => None,
    }
}

/// Flush accumulated candidates into `read_stmts`.
///
/// A run of one is emitted as a plain `p.read_T` call — no buffer allocation.
/// Two or more become a single [`gen_batch_block`].
fn flush_batch(batch_run: &mut Vec<BatchCandidate>, read_stmts: &mut Vec<TokenStream>) {
    if batch_run.is_empty() {
        return;
    }

    if batch_run.len() == 1 {
        read_stmts.push(single_candidate_stmt(batch_run.remove(0)));
    } else {
        let idx = read_stmts.len();
        read_stmts.push(gen_batch_block(batch_run, idx));
        batch_run.clear();
    }
}

/// Emit `let name: T = p.read_T(__base + off)?;` for a lone batch candidate.
///
/// Skips the buffer allocation that [`gen_batch_block`] would add — reading
/// one value through a single-element `[u8; N]` stack buffer is wasteful.
fn single_candidate_stmt(c: BatchCandidate) -> TokenStream {
    let name = &c.name;
    let field_ty = &c.field_ty;
    // Unsuffixed so it coerces to whatever addr type __base has.
    let off = Literal::i64_unsuffixed(c.offset as i64);
    let read_ts = if c.needs_from {
        let rfn = type_to_read_fn(&c.raw_ty).unwrap();
        quote! { #field_ty::from(p.#rfn(__base + #off)?) }
    } else {
        let rfn = type_to_read_fn(&c.field_ty).unwrap();
        quote! { p.#rfn(__base + #off)? }
    };
    quote! { let #name: #field_ty = #read_ts; }
}
