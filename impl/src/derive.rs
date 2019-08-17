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
    let linux_section = linker::linux::section(&ident);
    let macos_section = linker::macos::section(&ident);
    let windows_section = linker::windows::section(&ident);

    TokenStream::from(quote! {
        #[doc(hidden)]
        #[macro_export]
        macro_rules! #ident {
            ($item:item) => {
                #[used]
                #[cfg_attr(target_os = "linux", link_section = #linux_section)]
                #[cfg_attr(target_os = "macos", link_section = #macos_section)]
                #[cfg_attr(target_os = "windows", link_section = #windows_section)]
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
