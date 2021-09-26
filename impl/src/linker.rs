pub mod linux {
    use syn::Ident;

    pub fn section(ident: &Ident) -> String {
        format!("__libc_{}", ident)
    }

    pub fn section_start(ident: &Ident) -> String {
        format!("__start___libc_{}", ident)
    }

    pub fn section_stop(ident: &Ident) -> String {
        format!("__stop___libc_{}", ident)
    }
}

pub mod macos {
    use syn::Ident;

    // __libc_ prefix: see https://github.com/dtolnay/linkme/issues/41. This
    // makes recent versions of lld recognize the symbol as retained.
    pub fn section(ident: &Ident) -> String {
        format!("__DATA,__libc_{}", crate::hash(ident))
    }

    pub fn section_start(ident: &Ident) -> String {
        format!("\x01section$start$__DATA$__libc_{}", crate::hash(ident))
    }

    pub fn section_stop(ident: &Ident) -> String {
        format!("\x01section$end$__DATA$__libc_{}", crate::hash(ident))
    }
}

pub mod windows {
    use syn::Ident;

    pub fn section(ident: &Ident) -> String {
        format!(".linkme_{}$b", ident)
    }

    pub fn section_start(ident: &Ident) -> String {
        format!(".linkme_{}$a", ident)
    }

    pub fn section_stop(ident: &Ident) -> String {
        format!(".linkme_{}$c", ident)
    }
}

pub mod illumos {
    use syn::Ident;

    pub fn section(ident: &Ident) -> String {
        format!("set_linkme_{}", ident)
    }

    pub fn section_start(ident: &Ident) -> String {
        format!("__start_set_linkme_{}", ident)
    }

    pub fn section_stop(ident: &Ident) -> String {
        format!("__stop_set_linkme_{}", ident)
    }
}
