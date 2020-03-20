use syn::parse::{Error, ParseStream, Result};
use syn::Lit::*;
use syn::Meta::*;
use syn::{Attribute, Path};

pub(crate) fn extract_linkme_path(attrs: &mut Vec<Attribute>) -> Result<Path> {
    let mut linkme_path = None;

    let mut errors: Option<Error> = None;
    attrs.retain(|attr| {
        if attr.path.is_ident("linkme") {
            let res = attr.parse_args_with(|input: ParseStream| -> Result<Path> {
                match input.parse()? {
                    NameValue(m) if m.path.is_ident("crate") => match m.lit {
                        Str(lit) => lit.parse_with(Path::parse_mod_style),
                        lit => Err(Error::new_spanned(&lit, "expected a string literal")),
                    },
                    m => Err(Error::new_spanned(
                        &m,
                        "expected `#[linkme(crate = \"path::to::linkme\"]`",
                    )),
                }
            });

            match res {
                Ok(path) => {
                    linkme_path = Some(path);
                }
                Err(err) => match errors {
                    Some(ref mut errors) => errors.combine(err),
                    None => errors = Some(err),
                },
            }

            return false;
        }

        true
    });

    if let Some(errors) = errors {
        return Err(errors);
    }

    Ok(linkme_path.unwrap_or_else(|| syn::parse_quote!(::linkme)))
}
