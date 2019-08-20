use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::parse::{Parse, ParseStream, Result};
use syn::{bracketed, Attribute, Error, Ident, Token, Type, Visibility};

use crate::linker;

struct Declaration {
    attrs: Vec<Attribute>,
    vis: Visibility,
    ident: Ident,
    ty: Type,
}

impl Parse for Declaration {
    fn parse(input: ParseStream) -> Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let vis: Visibility = input.parse()?;
        input.parse::<Token![static]>()?;
        let ident: Ident = input.parse()?;
        input.parse::<Token![:]>()?;
        let ty: Type = input.parse()?;
        input.parse::<Token![=]>()?;

        let content;
        bracketed!(content in input);
        content.parse::<Token![..]>()?;

        input.parse::<Token![;]>()?;

        Ok(Declaration {
            attrs,
            vis,
            ident,
            ty,
        })
    }
}

pub fn expand(input: TokenStream) -> TokenStream {
    let msg = "distributed_slice is not implemented for this platform";
    let error = Error::new_spanned(&input, msg);
    let unsupported_platform = error.to_compile_error();

    let decl: Declaration = match syn::parse2(input) {
        Ok(decl) => decl,
        Err(err) => return err.to_compile_error(),
    };

    let attrs = decl.attrs;
    let vis = decl.vis;
    let ident = decl.ident;
    let ty = decl.ty;

    let linux_section = linker::linux::section(&ident);
    let linux_section_start = linker::linux::section_start(&ident);
    let linux_section_stop = linker::linux::section_stop(&ident);

    let macos_section_start = linker::macos::section_start(&ident);
    let macos_section_stop = linker::macos::section_stop(&ident);

    let windows_section_start = linker::windows::section_start(&ident);
    let windows_section_stop = linker::windows::section_stop(&ident);

    let call_site = Span::call_site();
    let ident_str = ident.to_string();
    let link_section_macro_dummy = format!("_linkme_macro_{}", ident);
    let link_section_macro_dummy = Ident::new(&link_section_macro_dummy, call_site);

    TokenStream::from(quote! {
        #(#attrs)*
        #vis static #ident: linkme::DistributedSlice<#ty> = {
            #[cfg(any(target_os = "linux", target_os = "macos"))]
            extern "C" {
                #[cfg_attr(target_os = "linux", link_name = #linux_section_start)]
                #[cfg_attr(target_os = "macos", link_name = #macos_section_start)]
                static LINKME_START: <#ty as linkme::private::Slice>::Element;

                #[cfg_attr(target_os = "linux", link_name = #linux_section_stop)]
                #[cfg_attr(target_os = "macos", link_name = #macos_section_stop)]
                static LINKME_STOP: <#ty as linkme::private::Slice>::Element;
            }

            #[cfg(target_os = "windows")]
            #[link_section = #windows_section_start]
            static LINKME_START: () = ();

            #[cfg(target_os = "windows")]
            #[link_section = #windows_section_stop]
            static LINKME_STOP: () = ();

            #[cfg(target_os = "linux")]
            #[link_section = #linux_section]
            #[used]
            static LINKME_PLEASE: [<#ty as linkme::private::Slice>::Element; 0] = [];

            #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
            #unsupported_platform

            unsafe {
                linkme::DistributedSlice::private_new(&LINKME_START, &LINKME_STOP)
            }
        };

        #[derive(linkme::link_section_macro)]
        #[doc(hidden)]
        #[linkme_ident = #ident_str]
        struct #link_section_macro_dummy;
    })
}
