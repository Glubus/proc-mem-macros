use super::field_attr::FieldAttr;
use super::parse::{parse_computed_attr, parse_nested_attr, parse_offset_attr};
use super::parse_ptr::parse_ptr_chain_attr;
use syn::Result;

/// Return the first `ReadMemory` attribute found on a field, if any.
///
/// Only the first match is returned; a second attribute on the same field is
/// silently ignored, consistent with how built-in derive macros behave.
/// Returns `Ok(None)` when no attribute is present — the caller emits the
/// "field must be annotated" error so the span points at the field itself.
pub fn extract_field_attr(attrs: &[syn::Attribute]) -> Result<Option<FieldAttr>> {
    for attr in attrs {
        let path = attr.path();
        if path.is_ident("offset") {
            return Ok(Some(attr.parse_args_with(parse_offset_attr)?));
        } else if path.is_ident("nested") {
            return Ok(Some(attr.parse_args_with(parse_nested_attr)?));
        } else if path.is_ident("ptr_chain") {
            return Ok(Some(attr.parse_args_with(parse_ptr_chain_attr)?));
        } else if path.is_ident("computed") {
            return Ok(Some(attr.parse_args_with(parse_computed_attr)?));
        } else if path.is_ident("skip") {
            return Ok(Some(FieldAttr::Skip));
        }
    }
    Ok(None)
}
