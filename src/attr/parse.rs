//! Parsers for `#[offset]`, `#[nested]`, and `#[computed]`.
//! The more complex `#[ptr_chain]` lives in [`super::parse_ptr`].

use super::field_attr::FieldAttr;
use syn::{parse::ParseStream, Expr, Ident, Result, Token, Type};

/// Parse `#[offset(EXPR [, via = TYPE])]`.
///
/// `via` is the intermediate primitive to read when the field type has no
/// direct `read_*` method (e.g. a newtype over `u8`); the macro calls
/// `FieldType::from(p.read_u8(addr)?)`.
pub fn parse_offset_attr(input: ParseStream) -> Result<FieldAttr> {
    let expr: Expr = input.parse()?;
    let mut via: Option<Type> = None;

    if input.peek(Token![,]) {
        input.parse::<Token![,]>()?;
        let key: Ident = input.parse()?;
        if key != "via" {
            return Err(syn::Error::new(key.span(), "expected `via` after offset"));
        }
        input.parse::<Token![=]>()?;
        via = Some(input.parse::<Type>()?);
    }
    Ok(FieldAttr::Offset { expr, via })
}

/// Parse `#[nested(EXPR)]` — offset at which a nested `ReadMemory` struct starts.
pub fn parse_nested_attr(input: ParseStream) -> Result<FieldAttr> {
    Ok(FieldAttr::Nested(input.parse()?))
}

/// Parse `#[computed(EXPR)]` — arbitrary expression emitted verbatim.
///
/// Can reference fields declared earlier in the struct because generated
/// code is top-to-bottom sequential.
pub fn parse_computed_attr(input: ParseStream) -> Result<FieldAttr> {
    Ok(FieldAttr::Computed(input.parse()?))
}
