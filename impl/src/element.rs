use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use std::iter::FromIterator;
use syn::parse::{Parse, ParseStream, Result};
use syn::{Attribute, Ident, Path, Token, Type, Visibility};

use crate::common::extract_linkme_path;

pub struct Element {
    attrs: Vec<Attribute>,
    vis: Visibility,
    ident: Ident,
    ty: Type,
    expr: TokenStream,
}

impl Parse for Element {
    fn parse(input: ParseStream) -> Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let vis: Visibility = input.parse()?;
        input.parse::<Token![static]>()?;
        let ident: Ident = input.parse()?;
        input.parse::<Token![:]>()?;
        let ty: Type = input.parse()?;
        input.parse::<Token![=]>()?;
        let mut expr_semi = Vec::from_iter(input.parse::<TokenStream>()?);
        if let Some(tail) = expr_semi.pop() {
            syn::parse2::<Token![;]>(TokenStream::from(tail))?;
        }
        let expr = TokenStream::from_iter(expr_semi);
        Ok(Element {
            attrs,
            vis,
            ident,
            ty,
            expr,
        })
    }
}

pub fn expand(path: Path, input: Element) -> TokenStream {
    let mut attrs = input.attrs;
    let vis = input.vis;
    let ident = input.ident;
    let ty = input.ty;
    let expr = input.expr;

    let linkme_path = match extract_linkme_path(&mut attrs) {
        Ok(path) => path,
        Err(err) => return err.to_compile_error(),
    };

    let span = quote!(#ty).into_iter().next().unwrap().span();
    let uninit = quote_spanned!(span=> #linkme_path::private::value::<#ty>());

    TokenStream::from(quote! {
        #path ! {
            #(#attrs)*
            #vis static #ident : #ty = {
                unsafe fn __typecheck(_: #linkme_path::private::Void) {
                    #linkme_path::DistributedSlice::private_typecheck(#path, #uninit)
                }

                #expr
            };
        }
    })
}
