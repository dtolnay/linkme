use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::{ParseStream, Parser, Result};
use syn::{DeriveInput, Ident, LitStr, Token};

use crate::linker;

pub fn expand(input: DeriveInput) -> TokenStream {
    let mut linkme_ident = None;
    for attr in input.attrs {
        if attr.path.is_ident("linkme_ident") {
            linkme_ident = parse_linkme_ident.parse2(attr.tts).ok();
        }
    }

    let ident = linkme_ident.expect("attribute linkme_ident");
    let section = linker::linux::section(&ident);

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

fn parse_linkme_ident(input: ParseStream) -> Result<Ident> {
    input.parse::<Token![=]>()?;
    let lit: LitStr = input.parse()?;
    lit.parse()
}
