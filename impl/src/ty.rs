use syn::{GenericArgument, Lifetime, PathArguments, Type};

pub(crate) fn populate_static_lifetimes(ty: &mut Type) {
    match ty {
        #![cfg_attr(all(test, exhaustive), deny(non_exhaustive_omitted_patterns))]
        Type::Array(ty) => populate_static_lifetimes(&mut ty.elem),
        Type::Group(ty) => populate_static_lifetimes(&mut ty.elem),
        Type::Paren(ty) => populate_static_lifetimes(&mut ty.elem),
        Type::Path(ty) => {
            if let Some(qself) = &mut ty.qself {
                populate_static_lifetimes(&mut qself.ty);
            }
            for segment in &mut ty.path.segments {
                if let PathArguments::AngleBracketed(segment) = &mut segment.arguments {
                    for arg in &mut segment.args {
                        if let GenericArgument::Type(arg) = arg {
                            populate_static_lifetimes(arg);
                        }
                    }
                }
            }
        }
        Type::Ptr(ty) => populate_static_lifetimes(&mut ty.elem),
        Type::Reference(ty) => {
            if ty.lifetime.is_none() {
                ty.lifetime = Some(Lifetime::new("'static", ty.and_token.span));
            }
            populate_static_lifetimes(&mut ty.elem);
        }
        Type::Slice(ty) => populate_static_lifetimes(&mut ty.elem),
        Type::Tuple(ty) => ty.elems.iter_mut().for_each(populate_static_lifetimes),
        Type::ImplTrait(_)
        | Type::Infer(_)
        | Type::Macro(_)
        | Type::Never(_)
        | Type::TraitObject(_)
        | Type::BareFn(_)
        | Type::Verbatim(_) => {}

        _ => unimplemented!("unknown Type"),
    }
}
