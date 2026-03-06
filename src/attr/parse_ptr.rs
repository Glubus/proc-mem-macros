use super::field_attr::FieldAttr;
use syn::{parse::ParseStream, Expr, Ident, Result, Token, Type};

/// Parse `#[ptr_chain(BASE, D1, …, FINAL [, via = TYPE])]`.
pub fn parse_ptr_chain_attr(input: ParseStream) -> Result<FieldAttr> {
    let base: Expr = input.parse()?;
    let (offsets, via) = parse_offsets_and_via(input)?;

    if offsets.is_empty() {
        return Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            "ptr_chain requires at least one offset",
        ));
    }

    Ok(FieldAttr::PtrChain { base, offsets, via })
}

/// Consume `, offset, …` entries until end-of-input or `, via = TYPE`.
fn parse_offsets_and_via(input: ParseStream) -> Result<(Vec<Expr>, Option<Type>)> {
    let mut offsets: Vec<Expr> = vec![];
    let mut via: Option<Type> = None;

    while input.peek(Token![,]) {
        input.parse::<Token![,]>()?;

        if input.is_empty() {
            break;
        }

        if let Some(ty) = try_parse_via(input)? {
            via = Some(ty);
            break;
        }

        offsets.push(input.parse::<Expr>()?);
    }

    Ok((offsets, via))
}

/// Try to parse `, via = TYPE` without consuming tokens if it isn't there.
///
/// Offsets are arbitrary expressions, so `via` (a plain identifier) is
/// ambiguous with a variable name.  We fork the stream to peek ahead: if the
/// next two tokens are `via =` we commit and parse the type; otherwise we
/// leave the stream untouched so the caller can parse it as an expression.
fn try_parse_via(input: ParseStream) -> Result<Option<Type>> {
    if !input.peek(Ident) {
        return Ok(None);
    }

    // Fork before consuming anything — we might need to backtrack.
    let fork = input.fork();
    let ident: Ident = fork.parse()?;

    if ident != "via" || !fork.peek(Token![=]) {
        return Ok(None);
    }

    // It really is `via =`: advance the real stream past `via =` and parse the type.
    input.parse::<Ident>()?;
    input.parse::<Token![=]>()?;
    Ok(Some(input.parse::<Type>()?))
}
