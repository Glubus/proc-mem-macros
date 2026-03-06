use syn::{Expr, Type};

/// One field's annotation, parsed from its attributes.
pub enum FieldAttr {
    /// `#[offset(EXPR [, via = TYPE])]` — reads a primitive at `__base + EXPR`.
    ///
    /// `via` is needed when the field is a newtype that doesn't have its own
    /// `read_*` method; the macro reads `via` and calls `FieldType::from(raw)`.
    Offset { expr: Expr, via: Option<Type> },

    /// `#[nested(EXPR)]` — calls `FieldType::read_from_memory(p [, state], __base + EXPR)?`.
    Nested(Expr),

    /// `#[ptr_chain(BASE, D1, …, FINAL [, via = TYPE])]`
    ///
    /// Each intermediate offset is a pointer dereference; only the last one
    /// (`FINAL`) is a plain read offset.  `offsets` must be non-empty.
    PtrChain {
        base: Expr,
        offsets: Vec<Expr>,
        via: Option<Type>,
    },

    /// `#[computed(EXPR)]` — emits `let field: T = EXPR;` verbatim.
    ///
    /// Can reference fields declared above it because generated code is sequential.
    Computed(Expr),

    /// `#[skip]` — fills the field with `Default::default()`.
    Skip,
}
