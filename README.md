## Linkme: safe cross-platform linker shenanigans

[![Build Status](https://api.travis-ci.com/dtolnay/linkme.svg?branch=master)](https://travis-ci.com/dtolnay/linkme)
[![Latest Version](https://img.shields.io/crates/v/linkme.svg)](https://crates.io/crates/linkme)
[![Rust Documentation](https://img.shields.io/badge/api-rustdoc-blue.svg)](https://docs.rs/linkme)

| Component | Linux | macOS | Windows | Other...<sup>†</sup> |
|:---|:---:|:---:|:---:|:---:|
| [Distributed slice] | :heavy\_check\_mark: | :heavy\_check\_mark: | :heavy\_check\_mark: | |

<b><sup>†</sup></b> We welcome PRs adding support for any platforms not listed
here.

[Distributed slice]: #distributed-slice

```toml
[dependencies]
linkme = "0.1"
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
   |                   ^^^^^ expected fn pointer, found usize
   |
   = note: expected type `fn(&mut other_crate::Bencher)`
              found type `usize`
```

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
