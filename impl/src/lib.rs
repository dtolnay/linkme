#![allow(clippy::needless_pass_by_value, clippy::too_many_lines)]

extern crate proc_macro;

mod args;
mod attr;
mod declaration;
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
        Args::Path(path) => element::expand(path, None, parse_macro_input!(input)),
        Args::PathPos(path, pos) => element::expand(path, pos, parse_macro_input!(input)),
    };

    TokenStream::from(expanded)
}
