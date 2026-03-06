use syn::{
    bracketed,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Expr, Ident, LitStr, Result, Token, Type,
};

/// Raw key-value bag parsed from `#[read_memory(...)]` or `#[read_memory_defaults(...)]`.
///
/// All fields are `Option` so either attribute can be partial; missing keys
/// fall back to the other layer in [`crate::config::StructConfig::resolve`].
#[derive(Default)]
pub struct ReadMemoryStructAttr {
    /// Reader type — `p: &'_ Reader` in the generated fn.
    pub reader: Option<Type>,
    /// If set, adds `state: &mut State` to the generated fn signature.
    pub state: Option<Type>,
    /// Error type returned by the generated fn.
    pub error: Option<Type>,
    /// Address integer type (default: `i32`).
    pub addr: Option<Type>,
    /// Reader method for pointer dereferences (default: `read_i32`).
    pub ptr_fn: Option<Ident>,
    /// Per-struct pointer-chain steps applied before reading fields.
    pub chain: Vec<Expr>,
    /// Rust statement injected verbatim before any reads (e.g. an early-return guard).
    pub guard: Option<LitStr>,
    /// Base-address expression; also triggers generation of the `read()` shorthand.
    pub init_base: Option<Expr>,
}

impl Parse for ReadMemoryStructAttr {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut attr = ReadMemoryStructAttr::default();
        while !input.is_empty() {
            parse_one_key(input, &mut attr)?;
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }
        Ok(attr)
    }
}

fn parse_one_key(input: ParseStream, attr: &mut ReadMemoryStructAttr) -> Result<()> {
    let key: Ident = input.parse()?;
    input.parse::<Token![=]>()?;

    match key.to_string().as_str() {
        "reader" => attr.reader = Some(input.parse::<Type>()?),
        "state" => attr.state = Some(input.parse::<Type>()?),
        "error" => attr.error = Some(input.parse::<Type>()?),
        "addr" => attr.addr = Some(input.parse::<Type>()?),
        "ptr_fn" => attr.ptr_fn = Some(input.parse::<Ident>()?),
        "chain" => attr.chain = parse_chain(input)?,
        "guard" => attr.guard = Some(input.parse::<LitStr>()?),
        "base" => attr.init_base = Some(input.parse::<Expr>()?),
        other => {
            return Err(syn::Error::new(
                key.span(),
                format!(
                    "unknown key `{other}`, expected one of: \
                     `reader`, `state`, `error`, `addr`, `ptr_fn`, `chain`, `guard`, `base`"
                ),
            ));
        }
    }

    Ok(())
}

fn parse_chain(input: ParseStream) -> Result<Vec<Expr>> {
    let content;
    bracketed!(content in input);
    let exprs = Punctuated::<Expr, Token![,]>::parse_terminated(&content)?;
    Ok(exprs.into_iter().collect())
}
