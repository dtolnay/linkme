#![feature(used_with_arg)]

mod slice {
    use linkme::distributed_slice;
    #[distributed_slice]
    static STATIC: [u32];
}

mod singleton {
    use linkme::disjointed_static;
    #[disjointed_static]
    static STATIC: u32;
    #[disjointed_static(STATIC)]
    static IMPL: u32 = 9;
}

fn main() {}
