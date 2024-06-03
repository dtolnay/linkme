use crate::{attr, ty};
use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned};
use syn::parse::{Error, Parse, ParseStream, Result};
use syn::{Attribute, Ident, Path, Token, Type, Visibility};

pub struct Item {
    attrs: Vec<Attribute>,
    vis: Visibility,
    ident: Ident,
    ty: Type,
    expr: TokenStream,
    orig_item: Option<TokenStream>,
    start_span: Span,
    end_span: Span,
}

impl Parse for Item {
    fn parse(input: ParseStream) -> Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let vis: Visibility = input.parse()?;
        input.parse::<Token![static]>()?;
        let mut_token: Option<Token![mut]> = input.parse()?;
        if let Some(mut_token) = mut_token {
            return Err(Error::new_spanned(
                mut_token,
                "static mut is not supported by disjointed_static",
            ));
        }
        let ident: Ident = input.parse()?;
        input.parse::<Token![:]>()?;
        let start_span = input.span();
        let ty: Type = input.parse()?;
        let end_span = quote!(#ty).into_iter().last().unwrap().span();
        input.parse::<Token![=]>()?;
        let mut expr_semi = Vec::from_iter(input.parse::<TokenStream>()?);
        if let Some(tail) = expr_semi.pop() {
            syn::parse2::<Token![;]>(TokenStream::from(tail))?;
        }
        let expr = TokenStream::from_iter(expr_semi);
        Ok(Item {
            attrs,
            vis,
            ident,
            ty,
            expr,
            orig_item: None,
            start_span,
            end_span,
        })
    }
}

pub fn expand(path: Path, input: Item) -> TokenStream {
    let mut attrs = input.attrs;
    let vis = input.vis;
    let ident = input.ident;
    let mut ty = input.ty;
    let expr = input.expr;
    let orig_item = input.orig_item;

    ty::populate_static_lifetimes(&mut ty);

    let linkme_path = match attr::linkme_path(&mut attrs) {
        Ok(path) => path,
        Err(err) => return err.to_compile_error(),
    };

    let factory = quote_spanned!(input.start_span=> __new);
    let get = quote_spanned!(input.end_span=> #factory());

    quote! {
        #path ! {
            #(#attrs)*
            #vis static #ident : #ty = {
                #[allow(clippy::no_effect_underscore_binding)]
                unsafe fn __typecheck(_: #linkme_path::__private::Void) {
                    let #factory = || -> fn() -> &'static #ty { || &#ident };
                    unsafe {
                        #linkme_path::DisjointedStatic::private_typecheck(#path, #get);
                    }
                }

                #expr
            };
        }

        #orig_item
    }
}
