// Copyright 2025 FastLabs Developers
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! # Traversable
//!
//! A visitor pattern implementation for traversing data structures.
//!
//! This crate provides [`Traversable`], [`TraversableMut`], and [`TraversableFold`] traits for
//! types that can be traversed, as well as [`Visitor`], [`VisitorMut`], and [`Folder`] traits for
//! types that perform the traversal.
//!
//! It is designed to be flexible and efficient, allowing for deep traversal of complex data
//! structures.
//!
//! ## Quick Start
//!
//! Add `traversable` to your `Cargo.toml` with the `derive` feature:
//!
//! ```toml
//! [dependencies]
//! traversable = { version = "0.3", features = ["derive", "std"] }
//! ```
//!
//! Define your data structures and derive [`Traversable`]:
//!
//! ```rust
//! # #[cfg(not(all(feature = "derive", feature = "std")))]
//! # fn main() {}
//! #
//! # #[cfg(all(feature = "derive", feature = "std"))]
//! # fn main() {
//! use std::any::Any;
//! use std::ops::ControlFlow;
//!
//! use traversable::Traversable;
//! use traversable::Visitor;
//!
//! #[derive(Traversable)]
//! struct Directory {
//!     name: String,
//!     files: Vec<File>,
//!     #[traverse(skip)]
//!     cache_id: u64,
//! }
//!
//! #[derive(Traversable)]
//! struct File {
//!     name: String,
//!     size: u64,
//! }
//!
//! struct FileCounter {
//!     count: usize,
//!     total_size: u64,
//! }
//!
//! impl Visitor for FileCounter {
//!     type Break = ();
//!
//!     fn enter(&mut self, node: &dyn Any) -> ControlFlow<Self::Break> {
//!         if let Some(file) = node.downcast_ref::<File>() {
//!             self.count += 1;
//!             self.total_size += file.size;
//!         }
//!         ControlFlow::Continue(())
//!     }
//! }
//!
//! let root = Directory {
//!     name: "root".to_string(),
//!     files: vec![
//!         File {
//!             name: "a.txt".to_string(),
//!             size: 100,
//!         },
//!         File {
//!             name: "b.rs".to_string(),
//!             size: 200,
//!         },
//!     ],
//!     cache_id: 12345,
//! };
//!
//! let mut counter = FileCounter {
//!     count: 0,
//!     total_size: 0,
//! };
//! root.traverse(&mut counter);
//!
//! assert_eq!(counter.count, 2);
//! assert_eq!(counter.total_size, 300);
//! # }
//! ```
//!
//! ## Attributes
//!
//! The derive macro supports the following attributes on structs and enums:
//!
//! * `#[traverse(skip_self)]`: Skips calling the visitor for the annotated type while still
//!   traversing its children.
//! * `#[traverse(skip_children)]`: Calls the visitor for the annotated type without traversing its
//!   children.
//!
//! The derive macro supports the following attributes on fields and variants:
//!
//! * `#[traverse(skip)]`: Skips traversing into the annotated field or variant.
//! * `#[traverse(with = "function_name")]`: Uses a custom function to traverse the field.
//!
//! ## Features
//!
//! * `derive`: Enables procedural macros `#[derive(Traversable)]`, `#[derive(TraversableMut)]`, and
//!   `#[derive(TraversableFold)]`.
//! * `std`: Enables support for standard library types (e.g., `Vec`, `HashMap`, `Box`).
//! * `traverse-trivial`: Enables traversal for primitive types (`u8`, `i32`, `bool`, etc.). By
//!   default, these are ignored.
//! * `traverse-std`: Enables traversal for "primary" std types like `String`. By default, these are
//!   ignored. Note that container types like `Vec` are always traversed if the `std` feature is
//!   enabled.

#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(missing_docs)]
#![no_std]

#[cfg(feature = "std")]
extern crate std;

use core::ops::ControlFlow;

#[cfg(feature = "derive")]
/// See [`Traversable`].
pub use traversable_derive::Traversable;
#[cfg(feature = "derive")]
/// See [`TraversableFold`].
pub use traversable_derive::TraversableFold;
#[cfg(feature = "derive")]
/// See [`TraversableMut`].
pub use traversable_derive::TraversableMut;

pub mod combinator;
pub mod function;

/// Implementations for third-party library types.
mod impls;

/// A visitor that can be used to traverse a data structure.
///
/// Implement this trait to define custom logic that executes when
/// [`Traversable`] items are visited. You can implement `enter` and `leave`
/// methods to perform actions before and after processing a node, respectively.
///
/// For an example of implementing `Visitor`, see the `FileCounter` struct
/// in the [crate-level documentation](self).
///
/// You can also use [`visitor`] to create a visitor from closures.
///
/// [`visitor`]: function::visitor
pub trait Visitor {
    /// The type that can be used to break traversal early.
    type Break;

    /// Called when the visitor is entering a node.
    ///
    /// Default implementation does nothing and continues traversal.
    fn enter(&mut self, this: &dyn core::any::Any) -> ControlFlow<Self::Break> {
        let _ = this;
        ControlFlow::Continue(())
    }

    /// Called when the visitor is leaving a node.
    ///
    /// Default implementation does nothing and continues traversal.
    fn leave(&mut self, this: &dyn core::any::Any) -> ControlFlow<Self::Break> {
        let _ = this;
        ControlFlow::Continue(())
    }
}

/// A visitor that can be used to traverse a mutable data structure.
///
/// Implement this trait to define custom logic that executes when
/// [`TraversableMut`] items are visited. You can implement `enter_mut` and `leave_mut`
/// methods to perform actions before and after processing a mutable node, respectively.
///
/// # Example
///
/// ```rust
/// # #[cfg(not(feature = "derive"))]
/// # fn main() {}
/// #
/// # #[cfg(feature = "derive")]
/// # fn main() {
/// use core::any::Any;
/// use core::ops::ControlFlow;
///
/// use traversable::TraversableMut;
/// use traversable::VisitorMut;
/// #[derive(TraversableMut)]
/// struct Node {
///     value: i32,
///     #[traverse(skip)]
///     id: u32,
/// }
///
/// struct Incrementer;
///
/// impl VisitorMut for Incrementer {
///     type Break = ();
///
///     fn enter_mut(&mut self, node: &mut dyn Any) -> ControlFlow<Self::Break> {
///         if let Some(n) = node.downcast_mut::<Node>() {
///             n.value += 1;
///         }
///         ControlFlow::Continue(())
///     }
/// }
///
/// let mut node = Node { value: 10, id: 1 };
/// node.traverse_mut(&mut Incrementer);
/// assert_eq!(node.value, 11);
/// # }
/// ```
///
/// You can also use [`visitor_mut`] to create a mutable visitor from closures.
///
/// [`visitor_mut`]: function::visitor_mut
pub trait VisitorMut {
    /// The type that can be used to break traversal early.
    type Break;

    /// Called when the visitor is entering a mutable node.
    ///
    /// Default implementation does nothing and continues traversal.
    fn enter_mut(&mut self, this: &mut dyn core::any::Any) -> ControlFlow<Self::Break> {
        let _ = this;
        ControlFlow::Continue(())
    }

    /// Called when the visitor is leaving a mutable node.
    ///
    /// Default implementation does nothing and continues traversal.
    fn leave_mut(&mut self, this: &mut dyn core::any::Any) -> ControlFlow<Self::Break> {
        let _ = this;
        ControlFlow::Continue(())
    }
}

/// A folder that can transform an owned data structure while traversing it.
///
/// Implement this trait to define custom logic that receives owned nodes and returns the node that
/// should continue through traversal. This is useful for bottom-up rewrites such as simplifying an
/// expression tree without using temporary replacement values.
///
/// [`TraversableFold`] calls [`Folder::enter`] before folding children and [`Folder::leave`] after
/// folding children. The default implementation returns each node unchanged.
///
/// You can also use [`folder`] to create a folder from closures.
///
/// [`folder`]: function::folder
pub trait Folder {
    /// The type that can be used to break traversal early.
    type Break;

    /// Called when the folder is entering an owned node.
    ///
    /// Default implementation returns the node unchanged and continues traversal.
    fn enter<T: core::any::Any>(&mut self, this: T) -> ControlFlow<Self::Break, T> {
        ControlFlow::Continue(this)
    }

    /// Called when the folder is leaving an owned node.
    ///
    /// Default implementation returns the node unchanged and continues traversal.
    fn leave<T: core::any::Any>(&mut self, this: T) -> ControlFlow<Self::Break, T> {
        ControlFlow::Continue(this)
    }
}

/// A trait for types that can be traversed by a visitor.
///
/// This trait is the core of the traversable pattern. It allows a [`Visitor`] to
/// walk through a data structure.
///
/// # Deriving `Traversable`
///
/// The easiest way to implement `Traversable` is to use the `derive` macro.
///
/// ```rust
/// # #[cfg(not(feature = "derive"))]
/// # fn main() {}
/// #
/// # #[cfg(feature = "derive")]
/// # fn main() {
/// use traversable::Traversable;
///
/// #[derive(Traversable)]
/// struct MyStruct {
///     data: u64,
///     #[traverse(skip)]
///     hidden: String,
/// }
/// # }
/// ```
///
/// # Attributes
///
/// The derive macro supports the following attributes on structs and enums:
///
/// * `#[traverse(skip_self)]`: Skips calling the visitor for the annotated type while still
///   traversing its children.
/// * `#[traverse(skip_children)]`: Calls the visitor for the annotated type without traversing its
///   children.
///
/// The derive macro supports the following attributes on fields and variants:
///
/// * `#[traverse(skip)]`: Skips traversing into the annotated field or variant.
/// * `#[traverse(with = "function_name")]`: Uses a custom function to traverse the field.
///
/// ## Custom Traversal Function
///
/// When using `#[traverse(with = "path::to::func")]`, the function must have the signature:
///
/// ```rust,ignore
/// fn func<V: Visitor>(item: &ItemType, visitor: &mut V) -> ControlFlow<V::Break>
/// ```
///
/// Example:
///
/// ```rust
/// # #[cfg(not(feature = "derive"))]
/// # fn main() {}
/// #
/// # #[cfg(feature = "derive")]
/// # fn main() {
/// use core::ops::ControlFlow;
///
/// use traversable::Traversable;
/// use traversable::Visitor;
///
/// fn traverse_string_len<V: Visitor>(s: &String, visitor: &mut V) -> ControlFlow<V::Break> {
///     s.len().traverse(visitor)
/// }
///
/// #[derive(Traversable)]
/// struct User {
///     #[traverse(with = "traverse_string_len")]
///     name: String,
/// }
/// # }
/// ```
pub trait Traversable: core::any::Any {
    /// Traverse the data structure with the given visitor.
    fn traverse<V: Visitor>(&self, visitor: &mut V) -> ControlFlow<V::Break>;
}

/// A trait for types that can be traversed mutably by a visitor.
///
/// This trait allows a [`VisitorMut`] to walk through a data structure and possibly
/// mutate it.
///
/// # Deriving `TraversableMut`
///
/// The easiest way to implement `TraversableMut` is to use the `derive` macro.
///
/// ```rust
/// # #[cfg(not(feature = "derive"))]
/// # fn main() {}
/// #
/// # #[cfg(feature = "derive")]
/// # fn main() {
/// use traversable::TraversableMut;
///
/// #[derive(TraversableMut)]
/// struct MyStruct {
///     data: u64,
///     #[traverse(skip)]
///     readonly: String,
/// }
/// # }
/// ```
///
/// # Attributes
///
/// The derive macro supports the following attributes on structs and enums:
///
/// * `#[traverse(skip_self)]`: Skips calling the visitor for the annotated type while still
///   traversing its children.
/// * `#[traverse(skip_children)]`: Calls the visitor for the annotated type without traversing its
///   children.
///
/// The derive macro supports the following attributes on fields and variants:
///
/// * `#[traverse(skip)]`: Skips traversing into the annotated field or variant.
/// * `#[traverse(with = "function_name")]`: Uses a custom function to traverse the field.
///
/// ## Custom Traversal Function
///
/// When using `#[traverse(with = "path::to::func")]`, the function must have the signature:
///
/// ```rust,ignore
/// fn func<V: VisitorMut>(item: &mut ItemType, visitor: &mut V) -> ControlFlow<V::Break>
/// ```
///
/// Example:
///
/// ```rust
/// # #[cfg(not(feature = "derive"))]
/// # fn main() {}
/// #
/// # #[cfg(feature = "derive")]
/// # fn main() {
/// use core::ops::ControlFlow;
///
/// use traversable::TraversableMut;
/// use traversable::VisitorMut;
///
/// fn traverse_string_chars<V: VisitorMut>(
///     s: &mut String,
///     visitor: &mut V,
/// ) -> ControlFlow<V::Break> {
///     // custom traversal logic
///     ControlFlow::Continue(())
/// }
///
/// #[derive(TraversableMut)]
/// struct User {
///     #[traverse(with = "traverse_string_chars")]
///     name: String,
/// }
/// # }
/// ```
pub trait TraversableMut: core::any::Any {
    /// Traverse the mutable data structure with the given visitor.
    fn traverse_mut<V: VisitorMut>(&mut self, visitor: &mut V) -> ControlFlow<V::Break>;
}

/// A trait for types that can be traversed and transformed by a folder.
///
/// This trait consumes `self`, folds its children, and returns the rebuilt value. It is intended
/// for owned transformations where a node may need to be replaced by another value of the same
/// type.
///
/// # Deriving `TraversableFold`
///
/// The easiest way to implement `TraversableFold` is to use the derive macro.
///
/// ```rust
/// # #[cfg(not(feature = "derive"))]
/// # fn main() {}
/// #
/// # #[cfg(feature = "derive")]
/// # fn main() {
/// use traversable::TraversableFold;
///
/// #[derive(TraversableFold)]
/// struct MyStruct {
///     data: u64,
///     #[traverse(skip)]
///     hidden: String,
/// }
/// # }
/// ```
///
/// # Attributes
///
/// The derive macro supports the following attributes on structs and enums:
///
/// * `#[traverse(skip_self)]`: Skips calling the folder for the annotated type while still folding
///   its children.
/// * `#[traverse(skip_children)]`: Calls the folder for the annotated type without folding its
///   children.
///
/// The derive macro supports the following attributes on fields and variants:
///
/// * `#[traverse(skip)]`: Skips folding into the annotated field or variant.
/// * `#[traverse(with = "function_name")]`: Uses a custom function to fold the field.
///
/// ## Custom Fold Function
///
/// When using `#[traverse(with = "path::to::func")]`, the function must have the signature:
///
/// ```rust,ignore
/// fn func<V: Folder>(item: ItemType, folder: &mut V) -> ControlFlow<V::Break, ItemType>
/// ```
pub trait TraversableFold: core::any::Any + Sized {
    /// Traverse and transform the data structure with the given folder.
    fn traverse_fold<V: Folder>(self, folder: &mut V) -> ControlFlow<V::Break, Self>;
}
