use syn::{Expr, ExprLit, Lit};

/// Extract a `u64` from a bare integer literal, or return `None`.
///
/// `None` for constants, paths, or arithmetic — the macro can't evaluate
/// those at expand time, so the field can't join a batch.
pub fn expr_as_u64(expr: &Expr) -> Option<u64> {
    if let Expr::Lit(ExprLit {
        lit: Lit::Int(li), ..
    }) = expr
    {
        li.base10_parse::<u64>().ok()
    } else {
        None
    }
}
