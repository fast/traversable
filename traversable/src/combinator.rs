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

//! Combinators for Visitors.

use crate::Visitor;
use crate::VisitorMut;

/// Extension trait for [`Visitor`]s.
pub trait VisitorExt: Visitor {
    /// Combines two visitors into one that runs them in sequence.
    ///
    /// If the first visitor returns [`ControlFlow::Break`], the combined visitor returns that break
    /// immediately. Otherwise, it runs the second visitor.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::ops::ControlFlow;
    ///
    /// use traversable::Traversable;
    /// use traversable::Visitor;
    /// use traversable::combinator::VisitorExt;
    ///
    /// #[derive(Traversable)]
    /// struct Person {
    ///     name: String,
    ///     age: u32,
    /// }
    ///
    /// struct LogVisitor;
    /// impl Visitor for LogVisitor {
    ///     type Break = ();
    ///     fn enter(&mut self, this: &dyn std::any::Any) -> ControlFlow<()> {
    ///         if let Some(s) = this.downcast_ref::<String>() {
    ///             println!("Visiting string: {}", s);
    ///         }
    ///         ControlFlow::Continue(())
    ///     }
    /// }
    ///
    /// let person = Person {
    ///     name: "Alice".to_string(),
    ///     age: 30,
    /// };
    ///
    /// let v1 = LogVisitor;
    /// let v2 = LogVisitor;
    ///
    /// // v1 runs first, then v2.
    /// let mut combined = v1.or(v2);
    /// person.traverse(&mut combined);
    /// ```
    fn or<V>(self, other: V) -> OrVisitor<Self, V>
    where
        Self: Sized,
        V: Visitor<Break = Self::Break>,
    {
        OrVisitor {
            visitor1: self,
            visitor2: other,
        }
    }
}

impl<V: Visitor> VisitorExt for V {}

/// Extension trait for [`VisitorMut`]s.
pub trait VisitorMutExt: VisitorMut {
    /// Combines two mutable visitors into one that runs them in sequence.
    ///
    /// If the first visitor returns [`ControlFlow::Break`], the combined visitor returns that break
    /// immediately. Otherwise, it runs the second visitor.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::ops::ControlFlow;
    ///
    /// use traversable::TraversableMut;
    /// use traversable::VisitorMut;
    /// use traversable::combinator::VisitorMutExt;
    ///
    /// #[derive(TraversableMut)]
    /// struct Data(i32);
    ///
    /// struct AddOne;
    /// impl VisitorMut for AddOne {
    ///     type Break = ();
    ///     fn enter_mut(&mut self, this: &mut dyn std::any::Any) -> ControlFlow<()> {
    ///         if let Some(d) = this.downcast_mut::<Data>() {
    ///             d.0 += 1;
    ///         }
    ///         ControlFlow::Continue(())
    ///     }
    /// }
    ///
    /// struct Double;
    /// impl VisitorMut for Double {
    ///     type Break = ();
    ///     fn enter_mut(&mut self, this: &mut dyn std::any::Any) -> ControlFlow<()> {
    ///         if let Some(d) = this.downcast_mut::<Data>() {
    ///             d.0 *= 2;
    ///         }
    ///         ControlFlow::Continue(())
    ///     }
    /// }
    ///
    /// let mut data = Data(1);
    /// // (1 + 1) * 2 = 4
    /// let mut combined = AddOne.or(Double);
    /// data.traverse_mut(&mut combined);
    /// assert_eq!(data.0, 4);
    /// ```
    fn or<V>(self, other: V) -> OrVisitor<Self, V>
    where
        Self: Sized,
        V: VisitorMut<Break = Self::Break>,
    {
        OrVisitor {
            visitor1: self,
            visitor2: other,
        }
    }
}

impl<V: VisitorMut> VisitorMutExt for V {}

/// A visitor that runs two visitors in sequence.
///
/// This struct is created by the [`or`](VisitorExt::or) method on [`VisitorExt`] or
/// [`VisitorMutExt`].
pub struct OrVisitor<V1, V2> {
    visitor1: V1,
    visitor2: V2,
}

impl<V1, V2> Visitor for OrVisitor<V1, V2>
where
    V1: Visitor,
    V2: Visitor<Break = V1::Break>,
{
    type Break = V1::Break;

    fn enter(&mut self, this: &dyn core::any::Any) -> core::ops::ControlFlow<Self::Break> {
        self.visitor1.enter(this)?;
        self.visitor2.enter(this)
    }

    fn leave(&mut self, this: &dyn core::any::Any) -> core::ops::ControlFlow<Self::Break> {
        self.visitor1.leave(this)?;
        self.visitor2.leave(this)
    }
}

impl<V1, V2> VisitorMut for OrVisitor<V1, V2>
where
    V1: VisitorMut,
    V2: VisitorMut<Break = V1::Break>,
{
    type Break = V1::Break;

    fn enter_mut(&mut self, this: &mut dyn core::any::Any) -> core::ops::ControlFlow<Self::Break> {
        self.visitor1.enter_mut(this)?;
        self.visitor2.enter_mut(this)
    }

    fn leave_mut(&mut self, this: &mut dyn core::any::Any) -> core::ops::ControlFlow<Self::Break> {
        self.visitor1.leave_mut(this)?;
        self.visitor2.leave_mut(this)
    }
}
