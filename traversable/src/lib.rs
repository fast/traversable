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
//! This crate provides a [`Traversable`] trait (and [`TraversableMut`]) for types that can be
//! traversed, and a [`Visitor`] trait (and [`VisitorMut`]) for types that perform the traversal.
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
//! traversable = { version = "0.1", features = ["derive", "std"] }
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
//!     #[traverse(skip)] // skip traverse this field
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
//! The derive macro supports the following attributes on fields and variants:
//!
//! * `#[traverse(skip)]`: Skips traversal of the annotated field or variant.
//! * `#[traverse(with = "function_name")]`: Uses a custom function to traverse the field.
//!
//! ## Features
//!
//! * `derive`: Enables procedural macros `#[derive(Traversable)]` and `#[derive(TraversableMut)]`.
//! * `std`: Enables support for standard library types (e.g., `Vec`, `HashMap`, `Box`).
//! * `traverse-trivial`: Enables traversal for primitive types (`u8`, `i32`, `bool`, etc.). By
//!   default, these are ignored.
//! * `traverse-std`: Enables traversal for "primary" std types like `String`. By default, these are
//!   ignored.

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
/// You can also use [`make_visitor`] to create a visitor from closures.
///
/// [`make_visitor`]: function::make_visitor
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
/// You can also use [`make_visitor_mut`] to create a mutable visitor from closures.
///
/// [`make_visitor_mut`]: function::make_visitor_mut
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
///     #[traverse(skip)] // skip this field
///     hidden: String,
/// }
/// # }
/// ```
///
/// # Attributes
///
/// The derive macro supports the following attributes:
///
/// * `#[traverse(skip)]`: Skips traversal of the annotated field or variant.
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
///     #[traverse(skip)] // skip this field
///     readonly: String,
/// }
/// # }
/// ```
///
/// # Attributes
///
/// The derive macro supports the following attributes:
///
/// * `#[traverse(skip)]`: Skips traversal of the annotated field or variant.
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

#[allow(unused_macros)]
macro_rules! blank_traverse_impl {
    ( $type:ty ) => {
        impl Traversable for $type {
            #[inline]
            fn traverse<V: Visitor>(&self, _visitor: &mut V) -> ControlFlow<V::Break> {
                ControlFlow::Continue(())
            }
        }

        impl TraversableMut for $type {
            #[inline]
            fn traverse_mut<V: VisitorMut>(&mut self, _visitor: &mut V) -> ControlFlow<V::Break> {
                ControlFlow::Continue(())
            }
        }
    };
}

#[allow(unused_macros)]
macro_rules! trivial_traverse_impl {
    ( $type:ty ) => {
        impl Traversable for $type {
            fn traverse<V: Visitor>(&self, visitor: &mut V) -> ControlFlow<V::Break> {
                visitor.enter(self)?;
                visitor.leave(self)?;
                ControlFlow::Continue(())
            }
        }

        impl TraversableMut for $type {
            fn traverse_mut<V: VisitorMut>(&mut self, visitor: &mut V) -> ControlFlow<V::Break> {
                visitor.enter_mut(self)?;
                visitor.leave_mut(self)?;
                ControlFlow::Continue(())
            }
        }
    };
}

mod impl_trivial {
    use super::*;

    #[cfg(not(feature = "traverse-trivial"))]
    macro_rules! trivial_impl {
        ( $type:ty ) => {
            blank_traverse_impl!($type);
        };
    }

    #[cfg(feature = "traverse-trivial")]
    macro_rules! trivial_impl {
        ( $type:ty ) => {
            trivial_traverse_impl!($type);
        };
    }

    trivial_impl!(());

    trivial_impl!(u8);
    trivial_impl!(u16);
    trivial_impl!(u32);
    trivial_impl!(u64);
    trivial_impl!(u128);
    trivial_impl!(usize);

    trivial_impl!(i8);
    trivial_impl!(i16);
    trivial_impl!(i32);
    trivial_impl!(i64);
    trivial_impl!(i128);
    trivial_impl!(isize);

    trivial_impl!(f32);
    trivial_impl!(f64);

    trivial_impl!(char);
    trivial_impl!(bool);
}

mod impl_tuple {
    use super::*;

    macro_rules! tuple_impl {
        ( $( $( $type:ident ),+ => $( $field:tt ),+ )+ ) => {
            $(
                impl<$( $type ),+> Traversable for ($($type,)+)
                where
                    $(
                        $type: Traversable
                    ),+
                {
                    fn traverse<V: Visitor>(&self, visitor: &mut V) -> ControlFlow<V::Break> {
                        $(
                            self.$field.traverse(visitor)?;
                        )+
                        ControlFlow::Continue(())
                    }
                }

                impl<$( $type ),+> TraversableMut for ($($type,)+)
                where
                    $(
                        $type: TraversableMut
                    ),+
                {
                    fn traverse_mut<V: VisitorMut>(&mut self, visitor: &mut V) -> ControlFlow<V::Break> {
                        $(
                            self.$field.traverse_mut(visitor)?;
                        )+
                        ControlFlow::Continue(())
                    }
                }
            )+
        };
    }

    tuple_impl! {
        T0 => 0
        T0, T1 => 0, 1
        T0, T1, T2 => 0, 1, 2
        T0, T1, T2, T3 => 0, 1, 2, 3
        T0, T1, T2, T3, T4 => 0, 1, 2, 3, 4
        T0, T1, T2, T3, T4, T5 => 0, 1, 2, 3, 4, 5
        T0, T1, T2, T3, T4, T5, T6 => 0, 1, 2, 3, 4, 5, 6
        T0, T1, T2, T3, T4, T5, T6, T7 => 0, 1, 2, 3, 4, 5, 6, 7
        T0, T1, T2, T3, T4, T5, T6, T7, T8 => 0, 1, 2, 3, 4, 5, 6, 7, 8
        T0, T1, T2, T3, T4, T5, T6, T7, T8, T9 => 0, 1, 2, 3, 4, 5, 6, 7, 8, 9
        T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10 => 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10
        T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11 => 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11
    }
}

#[cfg(feature = "std")]
mod impl_std_primary {
    use std::string::String;

    use super::*;

    #[cfg(not(feature = "traverse-std"))]
    macro_rules! std_primary_impl {
        ( $type:ty ) => {
            blank_traverse_impl!($type);
        };
    }

    #[cfg(feature = "traverse-std")]
    macro_rules! std_primary_impl {
        ( $type:ty ) => {
            trivial_traverse_impl!($type);
        };
    }

    std_primary_impl!(String);
}

#[cfg(feature = "std")]
mod impl_std_container {
    use std::boxed::Box;
    use std::cell::Cell;
    use std::sync::Arc;
    use std::sync::Mutex;
    use std::sync::RwLock;

    use super::*;

    // Helper traits to the generic `IntoIterator` Traversable impl
    trait DerefAndTraverse {
        fn deref_and_traverse<V: Visitor>(self, visitor: &mut V) -> ControlFlow<V::Break>;
    }

    trait DerefAndTraverseMut {
        fn deref_and_traverse_mut<V: VisitorMut>(self, visitor: &mut V) -> ControlFlow<V::Break>;
    }

    // Most collections iterate over item references, this is the trait impl that handles that case
    impl<T: Traversable> DerefAndTraverse for &T {
        fn deref_and_traverse<V: Visitor>(self, visitor: &mut V) -> ControlFlow<V::Break> {
            self.traverse(visitor)
        }
    }

    impl<T: TraversableMut> DerefAndTraverseMut for &mut T {
        fn deref_and_traverse_mut<V: VisitorMut>(self, visitor: &mut V) -> ControlFlow<V::Break> {
            self.traverse_mut(visitor)
        }
    }

    // Map-like collections iterate over item references pairs
    impl<TK: Traversable, TV: Traversable> DerefAndTraverse for (&TK, &TV) {
        fn deref_and_traverse<V: Visitor>(self, visitor: &mut V) -> ControlFlow<V::Break> {
            self.0.traverse(visitor)?;
            self.1.traverse(visitor)?;
            ControlFlow::Continue(())
        }
    }

    // Map-like collections have mutable iterators that allow mutating only the value, not the key
    impl<TK, TV: TraversableMut> DerefAndTraverseMut for (TK, &mut TV) {
        fn deref_and_traverse_mut<V: VisitorMut>(self, visitor: &mut V) -> ControlFlow<V::Break> {
            self.1.traverse_mut(visitor)
        }
    }

    // Implement Traversal for container types in standard library.
    macro_rules! impl_drive_for_into_iterator {
        ( $type:ty ; $($generics:tt)+ ) => {
            impl< $($generics)+ > Traversable for $type
            where
                $type: 'static,
                for<'a> &'a $type: IntoIterator,
                for<'a> <&'a $type as IntoIterator>::Item: DerefAndTraverse,
            {
                #[allow(for_loops_over_fallibles)]
                fn traverse<V: Visitor>(&self, visitor: &mut V) -> ControlFlow<V::Break> {
                    for item in self {
                        item.deref_and_traverse(visitor)?;
                    }
                    ControlFlow::Continue(())
                }
            }

            impl< $($generics)+ > TraversableMut for $type
            where
                $type: 'static,
                for<'a> &'a mut $type: IntoIterator,
                for<'a> <&'a mut $type as IntoIterator>::Item: DerefAndTraverseMut,
            {
                #[allow(for_loops_over_fallibles)]
                fn traverse_mut<V: VisitorMut>(&mut self, visitor: &mut V) -> ControlFlow<V::Break> {
                    for item in self {
                        item.deref_and_traverse_mut(visitor)?;
                    }
                    ControlFlow::Continue(())
                }
            }
        };
    }

    impl_drive_for_into_iterator! { [T] ; T }
    impl_drive_for_into_iterator! { [T; N] ; T, const N: usize }
    impl_drive_for_into_iterator! { std::vec::Vec<T> ; T }
    impl_drive_for_into_iterator! { std::collections::BTreeSet<T> ; T }
    impl_drive_for_into_iterator! { std::collections::BinaryHeap<T> ; T }
    impl_drive_for_into_iterator! { std::collections::HashSet<T> ; T }
    impl_drive_for_into_iterator! { std::collections::LinkedList<T> ; T }
    impl_drive_for_into_iterator! { std::collections::VecDeque<T> ; T }
    impl_drive_for_into_iterator! { std::collections::BTreeMap<T, U> ; T, U }
    impl_drive_for_into_iterator! { std::collections::HashMap<T, U> ; T, U }
    impl_drive_for_into_iterator! { Option<T> ; T }
    impl_drive_for_into_iterator! { Result<T, U> ; T, U }

    impl<T: Traversable> Traversable for Box<T> {
        fn traverse<V: Visitor>(&self, visitor: &mut V) -> ControlFlow<V::Break> {
            (**self).traverse(visitor)
        }
    }

    impl<T: TraversableMut> TraversableMut for Box<T> {
        fn traverse_mut<V: VisitorMut>(&mut self, visitor: &mut V) -> ControlFlow<V::Break> {
            (**self).traverse_mut(visitor)
        }
    }

    impl<T: Traversable> Traversable for Arc<T> {
        fn traverse<V: Visitor>(&self, visitor: &mut V) -> ControlFlow<V::Break> {
            (**self).traverse(visitor)
        }
    }

    impl<T> Traversable for Mutex<T>
    where
        T: Traversable,
    {
        fn traverse<V: Visitor>(&self, visitor: &mut V) -> ControlFlow<V::Break> {
            let lock = self.lock().unwrap();
            lock.traverse(visitor)
        }
    }

    impl<T> Traversable for RwLock<T>
    where
        T: Traversable,
    {
        fn traverse<V: Visitor>(&self, visitor: &mut V) -> ControlFlow<V::Break> {
            let lock = self.read().unwrap();
            lock.traverse(visitor)
        }
    }

    impl<T> TraversableMut for Arc<Mutex<T>>
    where
        T: TraversableMut,
    {
        fn traverse_mut<V: VisitorMut>(&mut self, visitor: &mut V) -> ControlFlow<V::Break> {
            let mut lock = self.lock().unwrap();
            lock.traverse_mut(visitor)
        }
    }

    impl<T> TraversableMut for Arc<RwLock<T>>
    where
        T: TraversableMut,
    {
        fn traverse_mut<V: VisitorMut>(&mut self, visitor: &mut V) -> ControlFlow<V::Break> {
            let mut lock = self.write().unwrap();
            lock.traverse_mut(visitor)
        }
    }

    impl<T> Traversable for Cell<T>
    where
        T: Traversable + Copy,
    {
        fn traverse<V: Visitor>(&self, visitor: &mut V) -> ControlFlow<V::Break> {
            self.get().traverse(visitor)
        }
    }

    impl<T> TraversableMut for Cell<T>
    where
        T: TraversableMut,
    {
        fn traverse_mut<V: VisitorMut>(&mut self, visitor: &mut V) -> ControlFlow<V::Break> {
            self.get_mut().traverse_mut(visitor)
        }
    }
}
