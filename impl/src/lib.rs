extern crate proc_macro;

mod args;
mod common;
mod declaration;
mod derive;
mod element;
mod linker;

use proc_macro::TokenStream;
use syn::parse_macro_input;

use crate::args::Args;

#[proc_macro_attribute]
pub fn distributed_slice(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as Args);

    let expanded = match args {
        Args::None => declaration::expand(parse_macro_input!(input)),
        Args::Path(path) => element::expand(path, parse_macro_input!(input)),
    };

    TokenStream::from(expanded)
}

#[doc(hidden)]
#[proc_macro_derive(link_section_macro, attributes(linkme_ident, linkme_macro))]
pub fn link_section_macro(input: TokenStream) -> TokenStream {
    let expanded = derive::expand(parse_macro_input!(input));
    TokenStream::from(expanded)
}
