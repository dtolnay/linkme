use syn::parse::{Error, ParseStream, Result};
use syn::{parse_quote, Attribute, Path, Token};

// #[linkme(crate = path::to::linkme)]
pub(crate) fn linkme_path(attrs: &mut Vec<Attribute>) -> Result<Path> {
    let mut linkme_path = None;
    let mut errors: Option<Error> = None;

    attrs.retain(|attr| {
        if !attr.path.is_ident("linkme") {
            return true;
        }
        match attr.parse_args_with(|input: ParseStream| {
            input.parse::<Token![crate]>()?;
            input.parse::<Token![=]>()?;
            input.call(Path::parse_mod_style)
        }) {
            Ok(path) => linkme_path = Some(path),
            Err(err) => match &mut errors {
                None => errors = Some(err),
                Some(errors) => errors.combine(err),
            },
        }
        false
    });

    match errors {
        None => Ok(linkme_path.unwrap_or_else(|| parse_quote!(::linkme))),
        Some(errors) => Err(errors),
    }
}
