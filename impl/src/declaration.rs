use crate::{attr, linker};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::parse::{Parse, ParseStream, Result};
use syn::{bracketed, Attribute, Error, Ident, Token, Type, Visibility};

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

    let mut attrs = decl.attrs;
    let vis = decl.vis;
    let ident = decl.ident;
    let ty = decl.ty;

    let linkme_path = match attr::linkme_path(&mut attrs) {
        Ok(path) => path,
        Err(err) => return err.to_compile_error(),
    };

    let linux_section = linker::linux::section(&ident);
    let linux_section_start = linker::linux::section_start(&ident);
    let linux_section_stop = linker::linux::section_stop(&ident);

    let macho_section_start = linker::macho::section_start(&ident);
    let macho_section_stop = linker::macho::section_stop(&ident);

    let windows_section_start = linker::windows::section_start(&ident);
    let windows_section_stop = linker::windows::section_stop(&ident);

    let illumos_section = linker::illumos::section(&ident);
    let illumos_section_start = linker::illumos::section_start(&ident);
    let illumos_section_stop = linker::illumos::section_stop(&ident);

    let freebsd_section = linker::freebsd::section(&ident);
    let freebsd_section_start = linker::freebsd::section_start(&ident);
    let freebsd_section_stop = linker::freebsd::section_stop(&ident);

    let call_site = Span::call_site();
    let ident_str = ident.to_string();
    let link_section_macro_dummy_str = format!("_linkme_macro_{}", ident);
    let link_section_macro_dummy = Ident::new(&link_section_macro_dummy_str, call_site);
    let link_section_enum_dummy_str = format!("_linkme_generate_{}", ident);
    let link_section_enum_dummy = Ident::new(&link_section_enum_dummy_str, call_site);

    quote! {
        #(#attrs)*
        #vis static #ident: #linkme_path::DistributedSlice<#ty> = {
            #[cfg(any(
                target_os = "none",
                target_os = "linux",
                target_os = "macos",
                target_os = "ios",
                target_os = "tvos",
                target_os = "illumos",
                target_os = "freebsd",
            ))]
            extern "C" {
                #[cfg_attr(any(target_os = "none", target_os = "linux"), link_name = #linux_section_start)]
                #[cfg_attr(any(target_os = "macos", target_os = "ios", target_os = "tvos"), link_name = #macho_section_start)]
                #[cfg_attr(target_os = "illumos", link_name = #illumos_section_start)]
                #[cfg_attr(target_os = "freebsd", link_name = #freebsd_section_start)]
                static LINKME_START: <#ty as #linkme_path::private::Slice>::Element;

                #[cfg_attr(any(target_os = "none", target_os = "linux"), link_name = #linux_section_stop)]
                #[cfg_attr(any(target_os = "macos", target_os = "ios", target_os = "tvos"), link_name = #macho_section_stop)]
                #[cfg_attr(target_os = "illumos", link_name = #illumos_section_stop)]
                #[cfg_attr(target_os = "freebsd", link_name = #freebsd_section_stop)]
                static LINKME_STOP: <#ty as #linkme_path::private::Slice>::Element;
            }

            #[cfg(target_os = "windows")]
            #[link_section = #windows_section_start]
            static LINKME_START: () = ();

            #[cfg(target_os = "windows")]
            #[link_section = #windows_section_stop]
            static LINKME_STOP: () = ();

            #[cfg(any(target_os = "none", target_os = "linux", target_os = "illumos", target_os = "freebsd"))]
            #[cfg_attr(any(target_os = "none", target_os = "linux"), link_section = #linux_section)]
            #[cfg_attr(target_os = "illumos", link_section = #illumos_section)]
            #[cfg_attr(target_os = "freebsd", link_section = #freebsd_section)]
            #[used]
            static mut LINKME_PLEASE: [<#ty as #linkme_path::private::Slice>::Element; 0] = [];

            #[cfg(not(any(
                target_os = "none",
                target_os = "linux",
                target_os = "macos",
                target_os = "ios",
                target_os = "tvos",
                target_os = "windows",
                target_os = "illumos",
                target_os = "freebsd",
            )))]
            #unsupported_platform

            #linkme_path::private::assert!(
                #linkme_path::private::mem::size_of::<<#ty as #linkme_path::private::Slice>::Element>() > 0,
            );

            unsafe {
                #linkme_path::DistributedSlice::private_new(&LINKME_START, &LINKME_STOP)
            }
        };

        #[doc(hidden)]
        #[allow(clippy::empty_enum)]
        #vis enum #link_section_macro_dummy {}

        #[doc(hidden)]
        #[derive(#linkme_path::link_section_macro)]
        enum #link_section_enum_dummy {
            _Ident = (#ident_str, 0).1,
            _Macro = (#link_section_macro_dummy_str, 1).1,
        }

        #[doc(hidden)]
        #vis use #link_section_macro_dummy as #ident;
    }
}
