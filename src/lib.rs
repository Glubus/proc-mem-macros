use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

mod attr;
mod config;
mod defaults;
mod expand;
mod utils;

/// Derive macro that generates a `read_from_memory` method on a struct.
///
/// # Required (per-struct or via `#[proc_mem_defaults]` on the enclosing module)
///
/// - `reader = YourProcess` — the type of `p` (your memory reader)
/// - `error = YourError` — the error type returned
///
/// # Optional
///
/// - `state = YourState` — adds `state: &mut YourState` to the signature; omit if unused
/// - `addr = i32` — address type (default: `i32`)
/// - `ptr_fn = read_ptr` — method called for pointer dereferences in `chain`/`ptr_chain` (default: `read_i32`)
/// - `chain = [0x10, 0x20]` — pointer walk steps applied to `base` before reading fields
/// - `guard = "expr"` — string literal emitted verbatim as a Rust statement (use `?` or `return Err(...)`)
/// - `base = expr` — expression that computes the base address; also generates a `read(p[, state])` convenience method
///
/// # Field attributes
///
/// Every field must have one of:
/// - `#[offset(expr)]` / `#[offset(expr, via = Type)]` — reads from `base + expr`
/// - `#[nested(expr)]` — calls `FieldType::read_from_memory(p[, state], base + expr)`
/// - `#[ptr_chain(base_expr, d1, d2, ..., final_offset)]` — independent pointer chain
/// - `#[computed(expr)]` — evaluates `expr` inline (may reference earlier fields)
/// - `#[skip]` — uses `Default::default()`
#[proc_macro_derive(
    ReadMemory,
    attributes(read_memory, read_memory_defaults, offset, nested, ptr_chain, computed, skip)
)]
pub fn derive_read_memory(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand::derive_read_memory(input)
        .unwrap_or_else(|e| e.to_compile_error().into())
        .into()
}

/// Module-level attribute that injects default configuration into every
/// `#[derive(ReadMemory)]` struct within the module.
///
/// Per-struct `#[read_memory(...)]` values always override the defaults.
///
/// # Example
///
/// ```rust,ignore
/// #[proc_mem_defaults(reader = MyProcess, state = MyState, error = MyError)]
/// mod structs {
///     use super::*;
///
///     #[derive(ReadMemory)]
///     #[read_memory(chain = [0x10], guard = "state.check_playing()?")]
///     pub struct Beatmap {
///         #[offset(0x1C)] pub id: i32,
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn proc_mem_defaults(attr: TokenStream, item: TokenStream) -> TokenStream {
    defaults::proc_mem_defaults_impl(attr.into(), item.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}
