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

//! Visitors from functions or closures.

use core::any::Any;
use core::marker::PhantomData;
use core::ops::ControlFlow;

use crate::Folder;
use crate::Visitor;
use crate::VisitorMut;

/// Type returned by `visitor` factories.
pub struct FnVisitor<T, B, F1, F2> {
    enter: F1,
    leave: F2,
    marker_type: PhantomData<T>,
    marker_break: PhantomData<B>,
}

/// Type returned by `folder` factories.
pub struct FnFolder<T, B, F1, F2> {
    enter: F1,
    leave: F2,
    marker_type: PhantomData<T>,
    marker_break: PhantomData<B>,
}

impl<T, B, F1, F2> Visitor for FnVisitor<T, B, F1, F2>
where
    T: Any,
    F1: FnMut(&T) -> ControlFlow<B>,
    F2: FnMut(&T) -> ControlFlow<B>,
{
    type Break = B;

    fn enter(&mut self, this: &dyn Any) -> ControlFlow<Self::Break> {
        if let Some(item) = this.downcast_ref::<T>() {
            (self.enter)(item)?;
        }
        ControlFlow::Continue(())
    }

    fn leave(&mut self, this: &dyn Any) -> ControlFlow<Self::Break> {
        if let Some(item) = this.downcast_ref::<T>() {
            (self.leave)(item)?;
        }
        ControlFlow::Continue(())
    }
}

impl<T, B, F1, F2> VisitorMut for FnVisitor<T, B, F1, F2>
where
    T: Any,
    F1: FnMut(&mut T) -> ControlFlow<B>,
    F2: FnMut(&mut T) -> ControlFlow<B>,
{
    type Break = B;

    fn enter_mut(&mut self, this: &mut dyn Any) -> ControlFlow<Self::Break> {
        if let Some(item) = this.downcast_mut::<T>() {
            (self.enter)(item)?;
        }
        ControlFlow::Continue(())
    }

    fn leave_mut(&mut self, this: &mut dyn Any) -> ControlFlow<Self::Break> {
        if let Some(item) = this.downcast_mut::<T>() {
            (self.leave)(item)?;
        }
        ControlFlow::Continue(())
    }
}

impl<T, B, F1, F2> Folder for FnFolder<T, B, F1, F2>
where
    T: Any,
    F1: FnMut(T) -> ControlFlow<B, T>,
    F2: FnMut(T) -> ControlFlow<B, T>,
{
    type Break = B;

    fn enter<U: Any>(&mut self, this: U) -> ControlFlow<Self::Break, U> {
        fold(this, &mut self.enter)
    }

    fn leave<U: Any>(&mut self, this: U) -> ControlFlow<Self::Break, U> {
        fold(this, &mut self.leave)
    }
}

fn fold<T, U, B, F>(this: U, fold: &mut F) -> ControlFlow<B, U>
where
    T: Any,
    U: Any,
    F: FnMut(T) -> ControlFlow<B, T>,
{
    use core::any::TypeId;
    use core::mem::ManuallyDrop;
    use core::mem::transmute_copy;

    if TypeId::of::<T>() != TypeId::of::<U>() {
        return ControlFlow::Continue(this);
    }

    // SAFETY: The `TypeId` check above proves that `T` and `U` are the same concrete `'static`
    // type. `ManuallyDrop` prevents the original `U` value from being dropped after its bits are
    // copied into the owned `T` value that is passed to the typed closure.
    let this = unsafe {
        let this = ManuallyDrop::new(this);
        transmute_copy(&this)
    };
    let this = fold(this)?;
    // SAFETY: The same `TypeId` equality still proves that `T` and `U` are the same concrete
    // `'static` type. `ManuallyDrop` prevents the closure result from being dropped after its bits
    // are copied back into the caller's expected `U` type.
    let this = unsafe {
        let this = ManuallyDrop::new(this);
        transmute_copy(&this)
    };
    ControlFlow::Continue(this)
}

type DefaultVisitFn<T, B> = fn(&T) -> ControlFlow<B>;
type DefaultVisitFnMut<T, B> = fn(&mut T) -> ControlFlow<B>;
type DefaultFoldFn<T, B> = fn(T) -> ControlFlow<B, T>;

/// Create a visitor that only visits items of a specific type from `enter` and `leave` closures.
///
/// This is a convenience function for creating simple visitors without defining
/// a new struct and implementing the [`Visitor`] trait manually.
///
/// # Example
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
/// use traversable::function::visitor;
///
/// #[derive(Traversable)]
/// struct Item {
///     value: i32,
/// }
///
/// let item = Item { value: 10 };
/// let mut total_value = 0;
///
/// let mut visitor = visitor::<Item, (), _, _>(
///     |node: &Item| {
///         // enter closure
///         total_value += node.value;
///         ControlFlow::Continue(())
///     },
///     |_node: &Item| {
///         // leave closure
///         ControlFlow::Continue(())
///     },
/// );
///
/// item.traverse(&mut visitor);
/// assert_eq!(total_value, 10);
/// # }
/// ```
pub fn visitor<T, B, F1, F2>(enter: F1, leave: F2) -> FnVisitor<T, B, F1, F2>
where
    T: Any,
    F1: FnMut(&T) -> ControlFlow<B>,
    F2: FnMut(&T) -> ControlFlow<B>,
{
    FnVisitor {
        enter,
        leave,
        marker_type: PhantomData,
        marker_break: PhantomData,
    }
}

/// Similar to [`visitor`], but the closure will only be called on entering.
///
/// # Example
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
/// use traversable::function::visitor_enter;
///
/// #[derive(Traversable)]
/// struct Item {
///     value: i32,
/// }
///
/// let item = Item { value: 20 };
/// let mut count = 0;
///
/// let mut visitor = visitor_enter::<Item, (), _>(|node: &Item| {
///     // enter closure
///     count += 1;
///     ControlFlow::Continue(())
/// });
///
/// item.traverse(&mut visitor);
/// assert_eq!(count, 1);
/// # }
/// ```
pub fn visitor_enter<T, B, F>(enter: F) -> FnVisitor<T, B, F, DefaultVisitFn<T, B>>
where
    T: Any,
    F: FnMut(&T) -> ControlFlow<B>,
{
    FnVisitor {
        enter,
        leave: |_| ControlFlow::Continue(()),
        marker_type: PhantomData,
        marker_break: PhantomData,
    }
}

/// Similar to [`visitor`], but the closure will only be called on leaving.
///
/// # Example
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
/// use traversable::function::visitor_leave;
///
/// #[derive(Traversable)]
/// struct Item {
///     value: i32,
/// }
///
/// let item = Item { value: 30 };
/// let mut visited_leave = false;
///
/// let mut visitor = visitor_leave::<Item, (), _>(|node: &Item| {
///     // leave closure
///     visited_leave = true;
///     ControlFlow::Continue(())
/// });
///
/// item.traverse(&mut visitor);
/// assert!(visited_leave);
/// # }
/// ```
pub fn visitor_leave<T, B, F>(leave: F) -> FnVisitor<T, B, DefaultVisitFn<T, B>, F>
where
    T: Any,
    F: FnMut(&T) -> ControlFlow<B>,
{
    FnVisitor {
        enter: |_| ControlFlow::Continue(()),
        leave,
        marker_type: PhantomData,
        marker_break: PhantomData,
    }
}

/// Create a visitor that only visits mutable items of a specific type from `enter` and `leave`
/// closures.
///
/// This is a convenience function for creating simple mutable visitors without defining
/// a new struct and implementing the [`VisitorMut`] trait manually.
///
/// # Example
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
/// use traversable::function::visitor_mut;
///
/// #[derive(TraversableMut)]
/// struct Item {
///     value: i32,
/// }
///
/// let mut item = Item { value: 10 };
///
/// let mut visitor = visitor_mut::<Item, (), _, _>(
///     |node: &mut Item| {
///         // enter_mut closure
///         node.value += 1;
///         ControlFlow::Continue(())
///     },
///     |_node: &mut Item| {
///         // leave_mut closure
///         ControlFlow::Continue(())
///     },
/// );
///
/// item.traverse_mut(&mut visitor);
/// assert_eq!(item.value, 11);
/// # }
/// ```
pub fn visitor_mut<T, B, F1, F2>(enter: F1, leave: F2) -> FnVisitor<T, B, F1, F2>
where
    T: Any,
    F1: FnMut(&mut T) -> ControlFlow<B>,
    F2: FnMut(&mut T) -> ControlFlow<B>,
{
    FnVisitor {
        enter,
        leave,
        marker_type: PhantomData,
        marker_break: PhantomData,
    }
}

/// Similar to [`visitor_mut`], but the closure will only be called on entering.
///
/// # Example
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
/// use traversable::function::visitor_enter_mut;
///
/// #[derive(TraversableMut)]
/// struct Item {
///     value: i32,
/// }
///
/// let mut item = Item { value: 20 };
/// let mut count = 0;
///
/// let mut visitor = visitor_enter_mut::<Item, (), _>(|node: &mut Item| {
///     // enter_mut closure
///     count += 1;
///     ControlFlow::Continue(())
/// });
///
/// item.traverse_mut(&mut visitor);
/// assert_eq!(count, 1);
/// # }
/// ```
pub fn visitor_enter_mut<T, B, F>(enter: F) -> FnVisitor<T, B, F, DefaultVisitFnMut<T, B>>
where
    T: Any,
    F: FnMut(&mut T) -> ControlFlow<B>,
{
    FnVisitor {
        enter,
        leave: |_| ControlFlow::Continue(()),
        marker_type: PhantomData,
        marker_break: PhantomData,
    }
}

/// Similar to [`visitor_mut`], but the closure will only be called on leaving.
///
/// # Example
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
/// use traversable::function::visitor_leave_mut;
///
/// #[derive(TraversableMut)]
/// struct Item {
///     value: i32,
/// }
///
/// let mut item = Item { value: 30 };
/// let mut visited_leave = false;
///
/// let mut visitor = visitor_leave_mut::<Item, (), _>(|node: &mut Item| {
///     // leave_mut closure
///     visited_leave = true;
///     ControlFlow::Continue(())
/// });
///
/// item.traverse_mut(&mut visitor);
/// assert!(visited_leave);
/// # }
/// ```
pub fn visitor_leave_mut<T, B, F>(leave: F) -> FnVisitor<T, B, DefaultVisitFnMut<T, B>, F>
where
    T: Any,
    F: FnMut(&mut T) -> ControlFlow<B>,
{
    FnVisitor {
        enter: |_| ControlFlow::Continue(()),
        leave,
        marker_type: PhantomData,
        marker_break: PhantomData,
    }
}

/// Create a folder that only folds items of a specific type from `enter` and `leave` closures.
///
/// This is a convenience function for creating simple owned transforms without defining a new
/// struct and implementing the [`Folder`] trait manually.
///
/// # Example
///
/// ```rust
/// # #[cfg(not(all(feature = "derive", feature = "std")))]
/// # fn main() {}
/// #
/// # #[cfg(all(feature = "derive", feature = "std"))]
/// # fn main() {
/// use core::ops::ControlFlow;
///
/// use traversable::TraversableFold;
/// use traversable::function::folder_leave;
///
/// #[derive(TraversableFold)]
/// enum Expr {
///     Add(Box<Expr>, Box<Expr>),
///     Literal(i32),
/// }
///
/// fn simplify(expr: Expr) -> Expr {
///     match expr {
///         Expr::Add(left, right) => match (*left, *right) {
///             (Expr::Literal(0), expr) | (expr, Expr::Literal(0)) => expr,
///             (left, right) => Expr::Add(Box::new(left), Box::new(right)),
///         },
///         expr => expr,
///     }
/// }
///
/// let expr = Expr::Add(Box::new(Expr::Literal(0)), Box::new(Expr::Literal(1)));
/// let mut folder = folder_leave::<Expr, (), _>(|expr| ControlFlow::Continue(simplify(expr)));
/// let expr = match expr.traverse_fold(&mut folder) {
///     ControlFlow::Continue(expr) => expr,
///     ControlFlow::Break(()) => unreachable!(),
/// };
///
/// assert!(matches!(expr, Expr::Literal(1)));
/// # }
/// ```
pub fn folder<T, B, F1, F2>(enter: F1, leave: F2) -> FnFolder<T, B, F1, F2>
where
    T: Any,
    F1: FnMut(T) -> ControlFlow<B, T>,
    F2: FnMut(T) -> ControlFlow<B, T>,
{
    FnFolder {
        enter,
        leave,
        marker_type: PhantomData,
        marker_break: PhantomData,
    }
}

/// Similar to [`folder`], but the closure will only be called on entering.
pub fn folder_enter<T, B, F>(enter: F) -> FnFolder<T, B, F, DefaultFoldFn<T, B>>
where
    T: Any,
    F: FnMut(T) -> ControlFlow<B, T>,
{
    FnFolder {
        enter,
        leave: ControlFlow::Continue,
        marker_type: PhantomData,
        marker_break: PhantomData,
    }
}

/// Similar to [`folder`], but the closure will only be called on leaving.
pub fn folder_leave<T, B, F>(leave: F) -> FnFolder<T, B, DefaultFoldFn<T, B>, F>
where
    T: Any,
    F: FnMut(T) -> ControlFlow<B, T>,
{
    FnFolder {
        enter: ControlFlow::Continue,
        leave,
        marker_type: PhantomData,
        marker_break: PhantomData,
    }
}
