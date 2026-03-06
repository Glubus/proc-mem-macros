use quote::format_ident;
use syn::Type;

/// Return the last path segment name of a simple type, or `None` for complex types.
///
/// Both `String` and `std::string::String` return `"String"` — only the final
/// segment matters for the `read_*` dispatch below.
pub fn type_name(ty: &Type) -> Option<String> {
    if let Type::Path(tp) = ty {
        tp.path.segments.last().map(|s| s.ident.to_string())
    } else {
        None
    }
}

/// Return the byte size of a batch-eligible primitive, or `None`.
///
/// `None` means the field can't join a batch and must be read individually.
pub fn primitive_byte_size(ty: &Type) -> Option<usize> {
    match type_name(ty)?.as_str() {
        "i8" | "u8" => Some(1),
        "i16" | "u16" => Some(2),
        "i32" | "u32" | "f32" => Some(4),
        "i64" | "u64" | "f64" => Some(8),
        _ => None,
    }
}

/// Map a primitive type to its `p.read_*` method name.
///
/// Returns `None` for non-primitive types — the caller should use `#[nested]`.
/// `String` is included because `read_string` is the conventional method name
/// in process-memory reader libraries.
pub fn type_to_read_fn(ty: &Type) -> Option<proc_macro2::Ident> {
    let method = match type_name(ty)?.as_str() {
        "i8" => "read_i8",
        "u8" => "read_u8",
        "i16" => "read_i16",
        "u16" => "read_u16",
        "i32" => "read_i32",
        "u32" => "read_u32",
        "i64" => "read_i64",
        "u64" => "read_u64",
        "f32" => "read_f32",
        "f64" => "read_f64",
        "String" => "read_string",
        _ => return None,
    };
    Some(format_ident!("{}", method))
}
