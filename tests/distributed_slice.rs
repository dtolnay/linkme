use linkme::distributed_slice;

#[distributed_slice]
static SHENANIGANS: [i32] = [..];

#[distributed_slice(SHENANIGANS)]
static N: i32 = 9;

#[distributed_slice(SHENANIGANS)]
static NN: i32 = 99;

#[distributed_slice(SHENANIGANS)]
static NNN: i32 = 999;

#[test]
fn test() {
    assert_eq!(SHENANIGANS.len(), 3);

    let mut sum = 0;
    for n in SHENANIGANS {
        sum += n;
    }

    assert_eq!(sum, 9 + 99 + 999);
}

#[test]
fn test_empty() {
    #[distributed_slice]
    static EMPTY: [i32] = [..];

    assert!(EMPTY.is_empty());
}

#[test]
fn test_non_copy() {
    struct NonCopy(i32);

    #[distributed_slice]
    static NONCOPY: [NonCopy] = [..];

    #[distributed_slice(NONCOPY)]
    static ELEMENT: NonCopy = NonCopy(9);

    assert!(!NONCOPY.is_empty());
}
