use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::{ParseStream, Parser, Result};
use syn::{DeriveInput, Ident, LitStr, Token};

pub fn expand(input: DeriveInput) -> TokenStream {
    let mut linkme_macro = None;
    let mut linkme_section = None;
    for attr in input.attrs {
        if attr.path.is_ident("linkme_macro") {
            linkme_macro = parse_linkme_macro.parse2(attr.tts).ok();
        } else if attr.path.is_ident("linkme_section") {
            linkme_section = parse_linkme_section.parse2(attr.tts).ok();
        }
    }

    let ident = linkme_macro.expect("attribute linkme_macro");
    let section = linkme_section.expect("attribute linkme_section");

    TokenStream::from(quote! {
        #[doc(hidden)]
        #[macro_export]
        macro_rules! #ident {
            ($item:item) => {
                #[used]
                #[link_section = #section]
                $item
            };
        }
    })
}

fn parse_linkme_macro(input: ParseStream) -> Result<Ident> {
    input.parse::<Token![=]>()?;
    let lit: LitStr = input.parse()?;
    lit.parse()
}

fn parse_linkme_section(input: ParseStream) -> Result<LitStr> {
    input.parse::<Token![=]>()?;
    input.parse()
}
