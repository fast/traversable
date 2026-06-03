# Visitor Pattern over Traversable Data Structures in Rust

[![Crates.io][crates-badge]][crates-url]
[![Documentation][docs-badge]][docs-url]
[![MSRV 1.85][msrv-badge]](https://www.whatrustisit.com)
[![Apache 2.0 licensed][license-badge]][license-url]
[![Build Status][actions-badge]][actions-url]

[crates-badge]: https://img.shields.io/crates/v/traversable.svg
[crates-url]: https://crates.io/crates/traversable
[docs-badge]: https://docs.rs/traversable/badge.svg
[msrv-badge]: https://img.shields.io/badge/MSRV-1.85-green?logo=rust
[docs-url]: https://docs.rs/traversable
[license-badge]: https://img.shields.io/crates/l/traversable
[license-url]: LICENSE
[actions-badge]: https://github.com/fast/traversable/workflows/CI/badge.svg
[actions-url]:https://github.com/fast/traversable/actions?query=workflow%3ACI

## Overview

This crate provides traits and proc macros to implement the visitor pattern for arbitrary data structures. This pattern is particularly useful when dealing with complex nested data structures, abstract trees and hierarchies of all kinds.

## Quick Start

Add `traversable` to your `Cargo.toml` with the `derive` feature:

```toml
[dependencies]
traversable = { version = "0.3", features = ["derive", "std"] }
```

Define your data structures and derive `Traversable`:

```rust
use std::any::Any;
use std::ops::ControlFlow;

use traversable::Traversable;
use traversable::Visitor;

#[derive(Traversable)]
struct Directory {
    name: String,
    files: Vec<File>,
    #[traverse(skip)]
    cache_id: u64,
}

#[derive(Traversable)]
struct File {
    name: String,
    size: u64,
}

struct FileCounter {
    count: usize,
    total_size: u64,
}

impl Visitor for FileCounter {
    type Break = ();

    fn enter(&mut self, node: &dyn Any) -> ControlFlow<Self::Break> {
        if let Some(file) = node.downcast_ref::<File>() {
            self.count += 1;
            self.total_size += file.size;
        }
        ControlFlow::Continue(())
    }
}

fn main() {
    let root = Directory {
        name: "root".to_string(),
        files: vec![
            File { name: "a.txt".to_string(), size: 100 },
            File { name: "b.rs".to_string(), size: 200 },
        ],
        cache_id: 12345,
    };

    let mut counter = FileCounter { count: 0, total_size: 0 };
    root.traverse(&mut counter);

    assert_eq!(counter.count, 2);
    assert_eq!(counter.total_size, 300);
}
```

## Attributes

The derive macro supports the following attributes on structs and enums:

*   `#[traverse(skip_self)]`: Skips calling the visitor for the annotated type while still traversing its children.
*   `#[traverse(skip_children)]`: Calls the visitor for the annotated type without traversing its children.

The derive macro supports the following attributes on fields and variants:

*   `#[traverse(skip)]`: Skips traversing into the annotated field or variant.
*   `#[traverse(with = "function_name")]`: Uses a custom function to traverse the field.

## Minimum Rust version policy

This crate is built against the latest stable release, and its minimum supported rustc version is 1.85.0.

The policy is that the minimum Rust version required to use this crate can be increased in minor version updates. For example, if Traversable 1.0 requires Rust 1.60.0, then Traversable 1.0.z for all values of z will also require Rust 1.60.0 or newer. However, Traversable 1.y for y > 0 may require a newer minimum version of Rust.

## License

This project is licensed under [Apache License, Version 2.0](LICENSE).
