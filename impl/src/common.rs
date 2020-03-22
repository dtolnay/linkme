use syn::ext::IdentExt;
use syn::parse::{Error, ParseStream, Result};
use syn::token::Paren;
use syn::{Attribute, Ident, LitStr, Path, Token};

pub(crate) fn extract_linkme_path(attrs: &mut Vec<Attribute>) -> Result<Path> {
    let mut linkme_path = None;

    let mut errors: Option<Error> = None;
    attrs.retain(|attr| {
        if attr.path.is_ident("linkme") {
            let res = attr.parse_args_with(|input: ParseStream| -> Result<Path> {
                let ident = input.call(Ident::parse_any)?;
                if ident == "crate" {
                    if input.peek(Paren) {
                        let content;
                        syn::parenthesized!(content in input);
                        content.call(Path::parse_mod_style)
                    } else {
                        let _: Token![=] = input.parse()?;
                        let lit: LitStr = input.parse()?;
                        lit.parse_with(Path::parse_mod_style)
                    }
                } else {
                    Err(Error::new_spanned(&ident, "unknown parameter"))
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
