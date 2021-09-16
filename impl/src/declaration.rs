use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::parse::{Parse, ParseStream, Result};
use syn::{bracketed, Attribute, Error, Ident, Token, Type, Visibility};

use crate::{attr, linker};

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

    let macos_section_start = linker::macos::section_start(&ident);
    let macos_section_stop = linker::macos::section_stop(&ident);

    let windows_section_start = linker::windows::section_start(&ident);
    let windows_section_stop = linker::windows::section_stop(&ident);

    let illumos_section = linker::illumos::section(&ident);
    let illumos_section_start = linker::illumos::section_start(&ident);
    let illumos_section_stop = linker::illumos::section_stop(&ident);

    let call_site = Span::call_site();
    let link_section_macro_str = format!("_linkme_macro_{}", ident);
    let link_section_macro = Ident::new(&link_section_macro_str, call_site);

    let declaration_macro = create_declaration_macro(&ident, &link_section_macro);

    quote! {
        #(#attrs)*
        #vis static #ident: #linkme_path::DistributedSlice<#ty> = {
            #[cfg(any(target_os = "none", target_os = "linux", target_os = "macos", target_os = "illumos"))]
            extern "C" {
                #[cfg_attr(any(target_os = "none", target_os = "linux"), link_name = #linux_section_start)]
                #[cfg_attr(target_os = "macos", link_name = #macos_section_start)]
                #[cfg_attr(target_os = "illumos", link_name = #illumos_section_start)]
                static LINKME_START: <#ty as #linkme_path::private::Slice>::Element;

                #[cfg_attr(any(target_os = "none", target_os = "linux"), link_name = #linux_section_stop)]
                #[cfg_attr(target_os = "macos", link_name = #macos_section_stop)]
                #[cfg_attr(target_os = "illumos", link_name = #illumos_section_stop)]
                static LINKME_STOP: <#ty as #linkme_path::private::Slice>::Element;
            }

            #[cfg(target_os = "windows")]
            #[link_section = #windows_section_start]
            static LINKME_START: () = ();

            #[cfg(target_os = "windows")]
            #[link_section = #windows_section_stop]
            static LINKME_STOP: () = ();

            #[cfg(any(target_os = "none", target_os = "linux", target_os = "illumos"))]
            #[cfg_attr(any(target_os = "none", target_os = "linux"), link_section = #linux_section)]
            #[cfg_attr(target_os = "illumos", link_section = #illumos_section)]
            #[used]
            static mut LINKME_PLEASE: [<#ty as #linkme_path::private::Slice>::Element; 0] = [];

            #[cfg(not(any(target_os = "none", target_os = "linux", target_os = "macos", target_os = "windows", target_os = "illumos")))]
            #unsupported_platform

            unsafe {
                #linkme_path::DistributedSlice::private_new(&LINKME_START, &LINKME_STOP)
            }
        };

        #declaration_macro

        #[doc(hidden)]
        #vis use #link_section_macro as #ident;
    }
}

fn create_declaration_macro(ident: &Ident, ident_macro: &Ident) -> TokenStream {
    let linux_section = linker::linux::section(ident);
    let macos_section = linker::macos::section(ident);
    let windows_section = linker::windows::section(ident);
    let illumos_section = linker::illumos::section(ident);

    quote! {
        #[doc(hidden)]
        #[macro_export]
        macro_rules! #ident_macro {
            (
                #![linkme_macro = $macro:path]
                #![linkme_sort_key = $key:tt]
                $item:item
            ) => {
                $macro ! {
                    #![linkme_linux_section = concat!(#linux_section, $key)]
                    #![linkme_macos_section = concat!(#macos_section, $key)]
                    #![linkme_windows_section = concat!(#windows_section, $key)]
                    #![linkme_illumos_section = concat!(#illumos_section, $key)]
                    $item
                }
            };
            (
                #![linkme_linux_section = $linux_section:expr]
                #![linkme_macos_section = $macos_section:expr]
                #![linkme_windows_section = $windows_section:expr]
                #![linkme_illumos_section = $illumos_section:expr]
                $item:item
            ) => {
                #[used]
                #[cfg_attr(any(target_os = "none", target_os = "linux"), link_section = $linux_section)]
                #[cfg_attr(target_os = "macos", link_section = $macos_section)]
                #[cfg_attr(target_os = "windows", link_section = $windows_section)]
                #[cfg_attr(target_os = "illumos", link_section = $illumos_section)]
                $item
            };
            ($item:item) => {
                #[used]
                #[cfg_attr(any(target_os = "none", target_os = "linux"), link_section = #linux_section)]
                #[cfg_attr(target_os = "macos", link_section = #macos_section)]
                #[cfg_attr(target_os = "windows", link_section = #windows_section)]
                #[cfg_attr(target_os = "illumos", link_section = #illumos_section)]
                $item
            };
        }
    }
}
