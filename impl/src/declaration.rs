use crate::{attr, linker, ty};
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
        let mut_token: Option<Token![mut]> = input.parse()?;
        if let Some(mut_token) = mut_token {
            return Err(Error::new_spanned(
                mut_token,
                "static mut is not supported by distributed_slice",
            ));
        }
        let ident: Ident = input.parse()?;
        input.parse::<Token![:]>()?;
        let ty: Type = input.parse()?;

        let eq_token: Option<Token![=]> = input.parse()?;
        if eq_token.is_some() {
            let content;
            bracketed!(content in input);
            content.parse::<Token![..]>()?;
        }

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
    let mut ty = decl.ty;
    let name = ident.to_string();

    let linkme_path = match attr::linkme_path(&mut attrs) {
        Ok(path) => path,
        Err(err) => return err.to_compile_error(),
    };

    ty::populate_static_lifetimes(&mut ty);

    let used = if cfg!(feature = "used_linker") {
        quote!(#[used(linker)])
    } else {
        quote!(#[used])
    };

    let linux_section = linker::linux::section(&ident);
    let linux_section_start = linker::linux::section_start(&ident);
    let linux_section_stop = linker::linux::section_stop(&ident);
    let linux_dupcheck = linux_section.replacen("linkme", "linkm2", 1);
    let linux_dupcheck_start = linux_section_start.replacen("linkme", "linkm2", 1);
    let linux_dupcheck_stop = linux_section_stop.replacen("linkme", "linkm2", 1);

    let macho_section = linker::macho::section(&ident);
    let macho_section_start = linker::macho::section_start(&ident);
    let macho_section_stop = linker::macho::section_stop(&ident);
    let macho_dupcheck = macho_section.replacen("linkme", "linkm2", 1);
    let macho_dupcheck_start = macho_section_start.replacen("linkme", "linkm2", 1);
    let macho_dupcheck_stop = macho_section_stop.replacen("linkme", "linkm2", 1);

    let windows_section = linker::windows::section(&ident);
    let windows_section_start = linker::windows::section_start(&ident);
    let windows_section_stop = linker::windows::section_stop(&ident);
    let windows_dupcheck = windows_section.replacen("linkme", "linkm2", 1);
    let windows_dupcheck_start = windows_section_start.replacen("linkme", "linkm2", 1);
    let windows_dupcheck_stop = windows_section_stop.replacen("linkme", "linkm2", 1);

    let illumos_section = linker::illumos::section(&ident);
    let illumos_section_start = linker::illumos::section_start(&ident);
    let illumos_section_stop = linker::illumos::section_stop(&ident);
    let illumos_dupcheck = illumos_section.replacen("linkme", "linkm2", 1);
    let illumos_dupcheck_start = illumos_section_start.replacen("linkme", "linkm2", 1);
    let illumos_dupcheck_stop = illumos_section_stop.replacen("linkme", "linkm2", 1);

    let bsd_section = linker::bsd::section(&ident);
    let bsd_section_start = linker::bsd::section_start(&ident);
    let bsd_section_stop = linker::bsd::section_stop(&ident);
    let bsd_dupcheck = bsd_section.replacen("linkme", "linkm2", 1);
    let bsd_dupcheck_start = bsd_section_start.replacen("linkme", "linkm2", 1);
    let bsd_dupcheck_stop = bsd_section_stop.replacen("linkme", "linkm2", 1);

    let call_site = Span::call_site();
    let link_section_macro_str = format!("_linkme_macro_{}", ident);
    let link_section_macro = Ident::new(&link_section_macro_str, call_site);

    let unsafe_extern = if cfg!(no_unsafe_extern_blocks) {
        None
    } else {
        Some(Token![unsafe](call_site))
    };

    let (unsafe_attr, link_section_attr) = if cfg!(no_unsafe_attributes) {
        // #[cfg_attr(all(), link_section = ...)]
        (
            Ident::new("cfg_attr", call_site),
            quote!(all(), link_section),
        )
    } else {
        // #[unsafe(link_section = ...)]
        (Ident::new("unsafe", call_site), quote!(link_section))
    };

    quote! {
        #(#attrs)*
        #vis static #ident: #linkme_path::DistributedSlice<#ty> = {
            #[cfg(any(
                target_os = "none",
                target_os = "linux",
                target_os = "macos",
                target_os = "ios",
                target_os = "tvos",
                target_os = "android",
                target_os = "fuchsia",
                target_os = "illumos",
                target_os = "freebsd",
                target_os = "openbsd",
                target_os = "psp",
            ))]
            #unsafe_extern extern "Rust" {
                #[cfg_attr(any(target_os = "none", target_os = "linux", target_os = "android", target_os = "fuchsia", target_os = "psp"), link_name = #linux_section_start)]
                #[cfg_attr(any(target_os = "macos", target_os = "ios", target_os = "tvos"), link_name = #macho_section_start)]
                #[cfg_attr(target_os = "illumos", link_name = #illumos_section_start)]
                #[cfg_attr(any(target_os = "freebsd", target_os = "openbsd"), link_name = #bsd_section_start)]
                static LINKME_START: <#ty as #linkme_path::__private::Slice>::Element;

                #[cfg_attr(any(target_os = "none", target_os = "linux", target_os = "android", target_os = "fuchsia", target_os = "psp"), link_name = #linux_section_stop)]
                #[cfg_attr(any(target_os = "macos", target_os = "ios", target_os = "tvos"), link_name = #macho_section_stop)]
                #[cfg_attr(target_os = "illumos", link_name = #illumos_section_stop)]
                #[cfg_attr(any(target_os = "freebsd", target_os = "openbsd"), link_name = #bsd_section_stop)]
                static LINKME_STOP: <#ty as #linkme_path::__private::Slice>::Element;

                #[cfg_attr(any(target_os = "none", target_os = "linux", target_os = "android", target_os = "fuchsia", target_os = "psp"), link_name = #linux_dupcheck_start)]
                #[cfg_attr(any(target_os = "macos", target_os = "ios", target_os = "tvos"), link_name = #macho_dupcheck_start)]
                #[cfg_attr(target_os = "illumos", link_name = #illumos_dupcheck_start)]
                #[cfg_attr(any(target_os = "freebsd", target_os = "openbsd"), link_name = #bsd_dupcheck_start)]
                static DUPCHECK_START: #linkme_path::__private::usize;

                #[cfg_attr(any(target_os = "none", target_os = "linux", target_os = "android", target_os = "fuchsia", target_os = "psp"), link_name = #linux_dupcheck_stop)]
                #[cfg_attr(any(target_os = "macos", target_os = "ios", target_os = "tvos"), link_name = #macho_dupcheck_stop)]
                #[cfg_attr(target_os = "illumos", link_name = #illumos_dupcheck_stop)]
                #[cfg_attr(any(target_os = "freebsd", target_os = "openbsd"), link_name = #bsd_dupcheck_stop)]
                static DUPCHECK_STOP: #linkme_path::__private::usize;
            }

            #[cfg(any(target_os = "uefi", target_os = "windows"))]
            #[#unsafe_attr(#link_section_attr = #windows_section_start)]
            static LINKME_START: [<#ty as #linkme_path::__private::Slice>::Element; 0] = [];

            #[cfg(any(target_os = "uefi", target_os = "windows"))]
            #[#unsafe_attr(#link_section_attr = #windows_section_stop)]
            static LINKME_STOP: [<#ty as #linkme_path::__private::Slice>::Element; 0] = [];

            #[cfg(any(target_os = "uefi", target_os = "windows"))]
            #[#unsafe_attr(#link_section_attr = #windows_dupcheck_start)]
            static DUPCHECK_START: () = ();

            #[cfg(any(target_os = "uefi", target_os = "windows"))]
            #[#unsafe_attr(#link_section_attr = #windows_dupcheck_stop)]
            static DUPCHECK_STOP: () = ();

            #used
            #[cfg(any(
                target_os = "none",
                target_os = "linux",
                target_os = "android",
                target_os = "fuchsia",
                target_os = "illumos",
                target_os = "freebsd",
                target_os = "openbsd",
                target_os = "psp",
            ))]
            #[cfg_attr(any(target_os = "none", target_os = "linux", target_os = "android", target_os = "fuchsia", target_os = "psp"), #unsafe_attr(#link_section_attr = #linux_section))]
            #[cfg_attr(target_os = "illumos", #unsafe_attr(#link_section_attr = #illumos_section))]
            #[cfg_attr(any(target_os = "freebsd", target_os = "openbsd"), #unsafe_attr(#link_section_attr = #bsd_section))]
            static mut LINKME_PLEASE: [<#ty as #linkme_path::__private::Slice>::Element; 0] = [];

            #used
            #[cfg_attr(any(target_os = "none", target_os = "linux", target_os = "android", target_os = "fuchsia", target_os = "psp"), #unsafe_attr(#link_section_attr = #linux_dupcheck))]
            #[cfg_attr(any(target_os = "macos", target_os = "ios", target_os = "tvos"), #unsafe_attr(#link_section_attr = #macho_dupcheck))]
            #[cfg_attr(any(target_os = "uefi", target_os = "windows"), #unsafe_attr(#link_section_attr = #windows_dupcheck))]
            #[cfg_attr(target_os = "illumos", #unsafe_attr(#link_section_attr = #illumos_dupcheck))]
            #[cfg_attr(any(target_os = "freebsd", target_os = "openbsd"), #unsafe_attr(#link_section_attr = #bsd_dupcheck))]
            static DUPCHECK: #linkme_path::__private::usize = 1;

            #[cfg(not(any(
                target_os = "none",
                target_os = "linux",
                target_os = "macos",
                target_os = "ios",
                target_os = "tvos",
                target_os = "windows",
                target_os = "uefi",
                target_os = "android",
                target_os = "fuchsia",
                target_os = "illumos",
                target_os = "freebsd",
                target_os = "openbsd",
                target_os = "psp",
            )))]
            #unsupported_platform

            #linkme_path::__private::assert!(
                #linkme_path::__private::mem::size_of::<<#ty as #linkme_path::__private::Slice>::Element>() > 0,
            );

            unsafe {
                #linkme_path::DistributedSlice::private_new(
                    #name,
                    #linkme_path::__private::ptr::addr_of!(LINKME_START),
                    #linkme_path::__private::ptr::addr_of!(LINKME_STOP),
                    #linkme_path::__private::ptr::addr_of!(DUPCHECK_START),
                    #linkme_path::__private::ptr::addr_of!(DUPCHECK_STOP),
                )
            }
        };

        #[doc(hidden)]
        #[macro_export]
        macro_rules! #link_section_macro {
            (
                #![linkme_macro = $macro:path]
                #![linkme_sort_key = $key:tt]
                $item:item
            ) => {
                $macro ! {
                    #![linkme_linux_section = concat!(#linux_section, $key)]
                    #![linkme_macho_section = concat!(#macho_section, $key)]
                    #![linkme_windows_section = concat!(#windows_section, $key)]
                    #![linkme_illumos_section = concat!(#illumos_section, $key)]
                    #![linkme_bsd_section = concat!(#bsd_section, $key)]
                    $item
                }
            };
            (
                #![linkme_linux_section = $linux_section:expr]
                #![linkme_macho_section = $macho_section:expr]
                #![linkme_windows_section = $windows_section:expr]
                #![linkme_illumos_section = $illumos_section:expr]
                #![linkme_bsd_section = $bsd_section:expr]
                $item:item
            ) => {
                #used
                #[cfg_attr(any(target_os = "none", target_os = "linux", target_os = "android", target_os = "fuchsia", target_os = "psp"), #unsafe_attr(#link_section_attr = $linux_section))]
                #[cfg_attr(any(target_os = "macos", target_os = "ios", target_os = "tvos"), #unsafe_attr(#link_section_attr = $macho_section))]
                #[cfg_attr(any(target_os = "uefi", target_os = "windows"), #unsafe_attr(#link_section_attr = $windows_section))]
                #[cfg_attr(target_os = "illumos", #unsafe_attr(#link_section_attr = $illumos_section))]
                #[cfg_attr(any(target_os = "freebsd", target_os = "openbsd"), #unsafe_attr(#link_section_attr = $bsd_section))]
                $item
            };
            ($item:item) => {
                #used
                #[cfg_attr(any(target_os = "none", target_os = "linux", target_os = "android", target_os = "fuchsia", target_os = "psp"), #unsafe_attr(#link_section_attr = #linux_section))]
                #[cfg_attr(any(target_os = "macos", target_os = "ios", target_os = "tvos"), #unsafe_attr(#link_section_attr = #macho_section))]
                #[cfg_attr(any(target_os = "uefi", target_os = "windows"), #unsafe_attr(#link_section_attr = #windows_section))]
                #[cfg_attr(target_os = "illumos", #unsafe_attr(#link_section_attr = #illumos_section))]
                #[cfg_attr(any(target_os = "freebsd", target_os = "openbsd"), #unsafe_attr(#link_section_attr = #bsd_section))]
                $item
            };
        }

        #[doc(hidden)]
        #vis use #link_section_macro as #ident;
    }
}
