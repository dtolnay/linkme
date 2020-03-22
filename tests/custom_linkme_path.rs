use linkme as link_me;

mod declaration {
    use crate::link_me::distributed_slice;

    #[distributed_slice]
    #[linkme(crate = "crate::link_me")]
    pub static SLICE: [i32] = [..];

    #[test]
    fn test_mod_slice() {
        assert!(!SLICE.is_empty());
    }
}

mod usage {
    use crate::link_me::distributed_slice;

    #[distributed_slice(super::declaration::SLICE)]
    #[linkme(crate(crate::link_me))]
    pub static N: i32 = 9;
}
