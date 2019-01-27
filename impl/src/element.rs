use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::parse::{Parse, ParseStream, Result};
use syn::{Attribute, Ident, Path, Token, Type, Visibility};

pub struct Element {
    attrs: Vec<Attribute>,
    vis: Visibility,
    ident: Ident,
    ty: Type,
    expr_semi: TokenStream,
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
        let expr_semi: TokenStream = input.parse()?;
        Ok(Element {
            attrs,
            vis,
            ident,
            ty,
            expr_semi,
        })
    }
}

pub fn expand(path: Path, input: Element) -> TokenStream {
    let attrs = input.attrs;
    let vis = input.vis;
    let ident = input.ident;
    let ty = input.ty;
    let expr_semi = input.expr_semi;

    let span = quote!(#ty).into_iter().next().unwrap().span();
    let uninit = quote_spanned!(span=> core::mem::uninitialized::<#ty>());

    TokenStream::from(quote! {
        #path ! {
            #(#attrs)*
            #vis static #ident : #ty = {
                unsafe fn __typecheck(_: linkme::private::Void) {
                    linkme::DistributedSlice::private_typecheck(#path, #uninit)
                }

                [#expr_semi 1][0]
            };
        }
    })
}
