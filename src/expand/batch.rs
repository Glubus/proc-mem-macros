//! Batch-read optimisation for consecutive `#[offset(LITERAL)]` fields.
//!
//! When multiple fields in a row have a statically-known offset and primitive
//! type, their bytes are read in one `p.read(base, total_len, &mut buf)?` call
//! instead of one call per field.  Each field then slices its bytes out of the
//! stack buffer with `from_le_bytes`.
//!
//! Eligibility requires: `#[offset(LITERAL)]` (not a runtime expression),
//! and a type whose byte size is known at macro-expand time (`i8`…`u64`,
//! `f32`, `f64`).  Any other field breaks the current run.

use proc_macro2::{Ident, Literal, TokenStream};
use quote::{format_ident, quote};
use syn::Type;

/// A field confirmed eligible for batch-reading.
pub struct BatchCandidate {
    pub name: Ident,
    /// Declared field type — used in the `let` binding.
    pub field_ty: Type,
    /// Primitive type actually read from memory (`field_ty` unless `via` was set).
    pub raw_ty: Type,
    /// Absolute byte offset from `__base`.
    pub offset: u64,
    /// Byte size of `raw_ty`.
    pub size: usize,
    /// Whether `FieldType::from(raw_val)` is needed after reading.
    pub needs_from: bool,
}

/// Emit one `let field = …from_le_bytes(buf[start..end]…)` binding.
fn gen_field_read(c: &BatchCandidate, min_off: u64, buf_name: &Ident) -> TokenStream {
    let name = &c.name;
    let field_ty = &c.field_ty;
    let raw_ty = &c.raw_ty;
    let rel_start = (c.offset - min_off) as usize;
    let rel_end = rel_start + c.size;

    if c.needs_from {
        quote! {
            let #name: #field_ty = #field_ty::from(
                #raw_ty::from_le_bytes(#buf_name[#rel_start..#rel_end].try_into().unwrap())
            );
        }
    } else {
        quote! {
            let #name: #field_ty = #field_ty::from_le_bytes(
                #buf_name[#rel_start..#rel_end].try_into().unwrap()
            );
        }
    }
}

/// Emit one `p.read(base, len, &mut buf)?` block covering all `candidates`.
///
/// `batch_idx` is appended to the buffer name so multiple batch blocks in the
/// same function don't shadow each other.
pub fn gen_batch_block(candidates: &[BatchCandidate], batch_idx: usize) -> TokenStream {
    let min_off = candidates.iter().map(|c| c.offset).min().unwrap();
    let max_end = candidates
        .iter()
        .map(|c| c.offset + c.size as u64)
        .max()
        .unwrap();
    let buf_size = (max_end - min_off) as usize;
    let buf_name = format_ident!("__batch_{}", batch_idx);
    // Unsuffixed so it coerces to whatever addr type __base has.
    let min_off_lit = Literal::i64_unsuffixed(min_off as i64);

    let field_reads: Vec<TokenStream> = candidates
        .iter()
        .map(|c| gen_field_read(c, min_off, &buf_name))
        .collect();

    quote! {
        let mut #buf_name = [0u8; #buf_size];
        p.read(__base + #min_off_lit, #buf_size, &mut #buf_name)?;
        #( #field_reads )*
    }
}
