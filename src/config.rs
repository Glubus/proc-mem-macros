use proc_macro2::{Ident, Span};
use quote::format_ident;
use syn::{parse_quote, Expr, LitStr, Result, Type};

use crate::attr::struct_attr::ReadMemoryStructAttr;

/// Fully resolved configuration used to drive code generation for one struct.
pub struct StructConfig {
    pub reader: Type,
    /// `None` means no `state` parameter in the generated fn.
    pub state: Option<Type>,
    pub error: Type,
    pub addr: Type,
    pub ptr_fn: Ident,
    pub chain: Vec<Expr>,
    pub guard: Option<LitStr>,
    pub init_base: Option<Expr>,
}

impl StructConfig {
    /// Merge per-struct and module-level default attributes into one config.
    ///
    /// Per-struct values win.  `chain`, `guard`, and `init_base` are
    /// struct-only — they cannot be inherited from module defaults because
    /// every struct has its own base address and pointer chain.
    ///
    /// # Errors
    ///
    /// Returns an error if `reader` or `error` is absent from both layers.
    pub fn resolve(
        per_struct: ReadMemoryStructAttr,
        defaults: ReadMemoryStructAttr,
        span: Span,
    ) -> Result<Self> {
        let reader = per_struct.reader.or(defaults.reader).ok_or_else(|| {
            syn::Error::new(
                span,
                "`reader` is required. Add `#[read_memory(reader = YourProcess)]` \
                 or `#[proc_mem_defaults(reader = YourProcess)]` on the module.",
            )
        })?;

        let error = per_struct.error.or(defaults.error).ok_or_else(|| {
            syn::Error::new(
                span,
                "`error` is required. Add `#[read_memory(error = YourError)]` \
                 or `#[proc_mem_defaults(error = YourError)]` on the module.",
            )
        })?;

        let state = per_struct.state.or(defaults.state);
        let addr = per_struct
            .addr
            .or(defaults.addr)
            .unwrap_or_else(|| parse_quote!(i32));
        let ptr_fn = per_struct
            .ptr_fn
            .or(defaults.ptr_fn)
            .unwrap_or_else(|| format_ident!("read_i32"));

        Ok(StructConfig {
            reader,
            state,
            error,
            addr,
            ptr_fn,
            chain: per_struct.chain,
            guard: per_struct.guard,
            init_base: per_struct.init_base,
        })
    }
}
