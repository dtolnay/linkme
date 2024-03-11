#![cfg_attr(all(test, exhaustive), feature(non_exhaustive_omitted_patterns_lint))]
#![allow(
    clippy::cast_possible_truncation, // https://github.com/rust-lang/rust-clippy/issues/7486
    clippy::needless_pass_by_value,
    clippy::too_many_lines,
    clippy::uninlined_format_args,
)]

mod args;
mod attr;
mod hash;
mod linker;
mod singleton;
mod slice;
mod ty;

use crate::args::Args;
use crate::hash::hash;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, quote_spanned};
use syn::parse_macro_input;
use syn::spanned::Spanned;

#[proc_macro_attribute]
pub fn distributed_slice(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as Args);

    let expanded = match args {
        Args::None => slice::declaration::expand(parse_macro_input!(input)),
        Args::Path(path) => slice::element::expand(path, None, parse_macro_input!(input)),
        Args::PathPos(path, pos) => slice::element::expand(path, pos, parse_macro_input!(input)),
    };

    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn disjointed_static(args: TokenStream, input: TokenStream) -> TokenStream {
    let args2: TokenStream2 = args.clone().into();
    let args = parse_macro_input!(args as Args);

    let expanded = match args {
        Args::None => singleton::declaration::expand(parse_macro_input!(input)),
        Args::Path(path) => singleton::item::expand(path, parse_macro_input!(input)),
        Args::PathPos(path, _) => {
            let sort = quote_spanned! {args2.span()=>
                compile_error!("disjointed_static does not accept a sort key");
            };
            let item = singleton::item::expand(path, parse_macro_input!(input));
            quote! {
                #sort
                #item
            }
        }
    };

    TokenStream::from(expanded)
}
