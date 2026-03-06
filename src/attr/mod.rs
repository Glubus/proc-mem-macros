//! Attribute parsing — turns raw `syn::Attribute` tokens into typed values.
//!
//! ```text
//! Field:   #[offset(EXPR [, via = TYPE])]
//!          #[nested(EXPR)]
//!          #[ptr_chain(BASE, D…, FINAL [, via = TYPE])]
//!          #[computed(EXPR)]
//!          #[skip]
//!
//! Struct:  #[read_memory(reader = T, error = E [, state = S] [, addr = A]
//!                        [, ptr_fn = f] [, chain = [O…]] [, guard = "stmt"]
//!                        [, base = EXPR])]
//! ```

pub mod extract;
pub mod field_attr;
pub mod parse;
pub mod parse_ptr;
pub mod struct_attr;

pub use extract::extract_field_attr;
pub use field_attr::FieldAttr;
pub use struct_attr::ReadMemoryStructAttr;
