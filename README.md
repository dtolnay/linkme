## Linkme: safe cross-platform linker shenanigans

[<img alt="github" src="https://img.shields.io/badge/github-dtolnay/linkme-8da0cb?style=for-the-badge&labelColor=555555&logo=github" height="20">](https://github.com/dtolnay/linkme)
[<img alt="crates.io" src="https://img.shields.io/crates/v/linkme.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/linkme)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-linkme-66c2a5?style=for-the-badge&labelColor=555555&logoColor=white&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K" height="20">](https://docs.rs/linkme)
[<img alt="build status" src="https://img.shields.io/github/workflow/status/dtolnay/linkme/CI/master?style=for-the-badge" height="20">](https://github.com/dtolnay/linkme/actions?query=branch%3Amaster)

| Component | Linux | macOS | Windows | Other...<sup>†</sup> |
|:---|:---:|:---:|:---:|:---:|
| [Distributed slice] | ✔️ | ✔️ | ✔️ | |

<b><sup>†</sup></b> We welcome PRs adding support for any platforms not listed
here.

[Distributed slice]: #distributed-slice

```toml
[dependencies]
linkme = "0.2"
```

*Supports rustc 1.32+*

<br>

# Distributed slice

A distributed slice is a collection of static elements that are gathered into a
contiguous section of the binary by the linker. Slice elements may be defined
individually from anywhere in the dependency graph of the final binary.

The implementation is based on `link_section` attributes and platform-specific
linker support. It does not involve life-before-main or any other runtime
initialization on any platform. This is a zero-cost safe abstraction that
operates entirely during compilation and linking.

### Declaration

A static distributed slice is declared by writing `#[distributed_slice]` on a
static item whose type is `[T]` for some type `T`. The initializer expression
must be `[..]` to indicate that elements come from elsewhere.

```rust
use linkme::distributed_slice;

#[distributed_slice]
pub static BENCHMARKS: [fn(&mut Bencher)] = [..];
```

### Elements

Slice elements may be registered into a distributed slice by a
`#[distributed_slice(...)]` attribute in which the path to the distributed slice
is given in the parentheses. The initializer is required to be a const
expression.

```rust
use linkme::distributed_slice;
use other_crate::BENCHMARKS;

#[distributed_slice(BENCHMARKS)]
static BENCH_DESERIALIZE: fn(&mut Bencher) = bench_deserialize;

fn bench_deserialize(b: &mut Bencher) {
    /* ... */
}
```

Elements may be defined in the same crate that declares the distributed slice,
or in any downstream crate. Elements across all crates linked into the final
binary will be observed to be present in the slice at runtime.

The distributed slice behaves in all ways like `&'static [T]`.

```rust
fn main() {
    // Iterate the elements.
    for bench in BENCHMARKS {
        /* ... */
    }

    // Index into the elements.
    let first = BENCHMARKS[0];

    // Slice the elements.
    let except_first = &BENCHMARKS[1..];

    // Invoke methods on the underlying slice.
    let len = BENCHMARKS.len();
}
```

The compiler will require that the static element type matches with the element
type of the distributed slice. If the two do not match, the program will not
compile:

```rust
#[distributed_slice(BENCHMARKS)]
static BENCH_WTF: usize = 999;
```

```console
error[E0308]: mismatched types
  --> src/distributed_slice.rs:65:19
   |
17 | static BENCH_WTF: usize = 999;
   |                   ^^^^^ expected fn pointer, found `usize`
   |
   = note: expected fn pointer `fn(&mut other_crate::Bencher)`
                    found type `usize`
```

### Function elements

As a shorthand for the common case of distributed slices containing function
pointers, the distributed\_slice attribute may be applied directly to a function
definition to place a pointer to that function into a distributed slice.

```rust
use linkme::distributed_slice;

#[distributed_slice]
pub static BENCHMARKS: [fn(&mut Bencher)] = [..];

// Equivalent to:
//
//    #[distributed_slice(BENCHMARKS)]
//    static _: fn(&mut Bencher) = bench_deserialize;
//
#[distributed_slice(BENCHMARKS)]
fn bench_deserialize(b: &mut Bencher) {
    /* ... */
}
```
###Renaming and re-exporting
When renaming or re-exporting the linkme library, the code generated in the macros will
still reference `::linkme` instead of the new name, requiring projects referencing your
code to also depend on linkme directly. T avoid that you can use the `#[linkme]` attribute
to tell linkme to generate code referencing your re-export. 

```rust
#[my_crate::linkme::distributed_slice]
#[linkme(crate=my_crate::linkme)] 
pub static BENCHMARKS: [i32] = [..];
```
Due to the way Rust macro expansion works there is no global setting for this, the
`#[linkme]` attribute must be repeated on every distributed slice element and on the
declaration.<br>
Also, note that the `#[linkme]` attribute itself does not have a path. This is because it isn't 
expanded as macro but consumed by the expansion of the `#[distributed_slice]` macro. 
Prefixing it with a path will break your build.   

<br>

#### License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
</sub>
