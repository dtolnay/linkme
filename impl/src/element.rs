use crate::attr;
use proc_macro2::TokenStream;
use quote::{format_ident, quote, quote_spanned};
use syn::parse::{Error, Parse, ParseStream, Result};
use syn::spanned::Spanned;
use syn::{
    Attribute, BareFnArg, BoundLifetimes, Expr, ExprPath, FnArg, GenericParam, Ident, Item, ItemFn,
    Pat, PatType, Path, Type, TypeBareFn, Visibility,
};

pub struct Element {
    attrs: Vec<Attribute>,
    vis: Visibility,
    ident: Ident,
    ty: Type,
    expr: Expr,
    orig_item: Option<Item>,
}

impl Parse for Element {
    fn parse(input: ParseStream) -> Result<Self> {
        match input.parse()? {
            Item::Static(item) => Ok(Element {
                attrs: item.attrs,
                vis: item.vis,
                ident: item.ident,
                ty: *item.ty,
                expr: *item.expr,
                orig_item: None,
            }),
            Item::Fn(item) => {
                let ident = format_ident!("_LINKME_ELEMENT_{}", item.sig.ident);
                let ty = extract_bare_fn_type(&item).map(Type::BareFn)?;
                let expr = Expr::Path(ExprPath {
                    attrs: vec![],
                    qself: None,
                    path: item.sig.ident.clone().into(),
                });

                Ok(Element {
                    attrs: vec![syn::parse_quote!(#[allow(non_upper_case_globals)])],
                    vis: Visibility::Inherited,
                    ident,
                    ty,
                    expr,
                    orig_item: Some(Item::Fn(item)),
                })
            }
            item => Err(Error::new_spanned(
                &item,
                "distributed element must be either static or function item",
            )),
        }
    }
}

pub fn expand(path: Path, input: Element) -> TokenStream {
    let mut attrs = input.attrs;
    let vis = input.vis;
    let ident = input.ident;
    let ty = input.ty;
    let expr = input.expr;
    let orig_item = input.orig_item;

    let linkme_path = match attr::linkme_path(&mut attrs) {
        Ok(path) => path,
        Err(err) => return err.to_compile_error(),
    };

    let span = match orig_item {
        Some(ref item) => item.span(),
        None => quote!(#ty).into_iter().next().unwrap().span(),
    };
    let new = quote_spanned!(span=> __new);
    let uninit = quote_spanned!(span=> __new());

    TokenStream::from(quote! {
        #path ! {
            #(#attrs)*
            #vis static #ident : #ty = {
                unsafe fn __typecheck(_: #linkme_path::private::Void) {
                    let #new = #linkme_path::private::value::<#ty>;
                    #linkme_path::DistributedSlice::private_typecheck(#path, #uninit)
                }

                #expr
            };
        }

        #orig_item
    })
}

fn extract_bare_fn_type(item: &ItemFn) -> Result<TypeBareFn> {
    let sig = &item.sig;

    if let Some(ref where_clause) = sig.generics.where_clause {
        for pred in &where_clause.predicates {
            if let syn::WherePredicate::Lifetime(lt) = pred {
                return Err(Error::new_spanned(
                    lt,
                    "lifetime bounds cannot be used in this context",
                ));
            }
        }
    }

    let mut lifetimes: Option<BoundLifetimes> = None;
    for param in &sig.generics.params {
        match param {
            GenericParam::Lifetime(lifetime) => {
                let lifetimes = lifetimes.get_or_insert_with(Default::default);
                lifetimes.lifetimes.push(lifetime.clone());
            }
            param => {
                return Err(Error::new_spanned(
                    param,
                    "distributed element cannot accept generic parameters",
                ));
            }
        }
    }

    let inputs = sig
        .inputs
        .iter()
        .map(|input| {
            let error =
                || Error::new_spanned(item, "methods cannot be specified as distributed element");

            match input {
                FnArg::Receiver(..) => Err(error()),
                // Note: the guard below is equivalent to
                //
                //     matches!(&**pat, Pat::Ident(pat) if pat.ident == "self")
                //
                // from 1.42.0.
                FnArg::Typed(PatType { ref pat, .. })
                    if match &**pat {
                        Pat::Ident(pat) if pat.ident == "self" => true,
                        _ => false,
                    } =>
                {
                    Err(error())
                }
                FnArg::Typed(ref pat) => Ok(BareFnArg {
                    attrs: pat.attrs.clone(),
                    name: None,
                    ty: (*pat.ty).clone(),
                }),
            }
        })
        .collect::<Result<_>>()?;

    Ok(TypeBareFn {
        lifetimes,
        unsafety: sig.unsafety.clone(),
        abi: sig.abi.clone(),
        fn_token: sig.fn_token.clone(),
        paren_token: sig.paren_token.clone(),
        inputs,
        variadic: sig.variadic.clone(),
        output: sig.output.clone(),
    })
}
