#![cfg_attr(feature = "used_linker", feature(used_with_arg))]

use linkme::distributed_slice;

pub struct Item {
    pub name: &'static str,
}

impl Item {
    #[inline(never)]
    fn len(&self) -> usize {
        self.name.len()
    }
}

#[distributed_slice]
static ITEMS: [Item];

#[distributed_slice(ITEMS)]
static ITEM1: Item = Item { name: "item1" };

// Regression test for https://github.com/dtolnay/linkme/issues/67.
//
// Must be run with `--release`.
#[test]
fn win_status_illegal_instruction() {
    let mut last_address = None;
    for item in ITEMS {
        // Do some busy work to push the compiler into optimizing the code in a
        // particularly "bad" way. This is derived from experimentation.
        let address = item as *const Item as usize;
        if let Some(last) = last_address {
            assert_eq!(address - last, std::mem::size_of::<Item>());
        }
        last_address = Some(address);
        println!("{} {:?}", item.len(), item.name);

        // Should not cause STATUS_ILLEGAL_INSTRUCTION.
        assert_eq!(item.len(), 5);
    }
}
