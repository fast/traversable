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

use core::any::Any;
use core::marker::PhantomData;
use core::ops::ControlFlow;
use std::vec::Vec;

use crate::Continue;
use crate::Visitor;
use crate::VisitorMut;

/// Events for a typed visitor with state scoped to each visited node.
pub enum VisitEvent<'a, S> {
    /// Enter a node with its enclosing matched node's state, or None at a root.
    Enter(Option<&'a S>),
    /// Leave a node with the state returned by its Enter callback.
    Leave(S),
}

/// Create a typed visitor whose Enter callback returns state for its children and Leave.
///
/// Returning [`Continue::Skip`] suppresses descendants and Leave for that node.
/// The continuation returned on Leave is ignored. Discard the visitor if traversal breaks,
/// since ancestors do not receive Leave events during early termination.
pub fn visitor_events<T: Any, S, B>(
    visit: impl FnMut(&T, VisitEvent<'_, S>) -> ControlFlow<B, Continue<S>>,
) -> impl Visitor<Break = B> {
    EventVisitor {
        visit,
        states: Vec::new(),
        marker: PhantomData::<fn(T) -> B>,
    }
}

/// Mutable counterpart of [`visitor_events`], with the same state and skipping semantics.
pub fn visitor_events_mut<T: Any, S, B>(
    visit: impl FnMut(&mut T, VisitEvent<'_, S>) -> ControlFlow<B, Continue<S>>,
) -> impl VisitorMut<Break = B> {
    EventVisitor {
        visit,
        states: Vec::new(),
        marker: PhantomData::<fn(T) -> B>,
    }
}

/// Create an enter-only visitor with state scoped to each matched node.
///
/// The callback receives the enclosing matched node's state, or None at each root.
/// Returning [`Continue::Children`] supplies state for its descendants; [`Continue::Skip`]
/// skips the subtree. [`ControlFlow::Break`] terminates the traversal.
pub fn visitor_enter_scoped<T: Any, S, B>(
    mut visit: impl FnMut(&T, Option<&S>) -> ControlFlow<B, Continue<S>>,
) -> impl Visitor<Break = B> {
    visitor_events(move |node, event| match event {
        VisitEvent::Enter(parent) => visit(node, parent),
        VisitEvent::Leave(_) => ControlFlow::Continue(Continue::Skip),
    })
}

/// Mutable counterpart of [`visitor_enter_scoped`].
pub fn visitor_enter_scoped_mut<T: Any, S, B>(
    mut visit: impl FnMut(&mut T, Option<&S>) -> ControlFlow<B, Continue<S>>,
) -> impl VisitorMut<Break = B> {
    visitor_events_mut(move |node, event| match event {
        VisitEvent::Enter(parent) => visit(node, parent),
        VisitEvent::Leave(_) => ControlFlow::Continue(Continue::Skip),
    })
}

struct EventVisitor<T, S, B, F> {
    visit: F,
    states: Vec<S>,
    marker: PhantomData<fn(T) -> B>,
}

impl<T: Any, S, B, F> Visitor for EventVisitor<T, S, B, F>
where
    F: FnMut(&T, VisitEvent<'_, S>) -> ControlFlow<B, Continue<S>>,
{
    type Break = B;

    fn enter(&mut self, node: &dyn Any) -> ControlFlow<B, Continue> {
        if let Some(node) = node.downcast_ref::<T>() {
            match (self.visit)(node, VisitEvent::Enter(self.states.last()))? {
                Continue::Children(state) => self.states.push(state),
                Continue::Skip => return ControlFlow::Continue(Continue::Skip),
            }
        }
        ControlFlow::Continue(Continue::Children(()))
    }

    fn leave(&mut self, node: &dyn Any) -> ControlFlow<B> {
        if let Some(node) = node.downcast_ref::<T>() {
            (self.visit)(node, VisitEvent::Leave(self.states.pop().unwrap()))?;
        }
        ControlFlow::Continue(())
    }
}

impl<T: Any, S, B, F> VisitorMut for EventVisitor<T, S, B, F>
where
    F: FnMut(&mut T, VisitEvent<'_, S>) -> ControlFlow<B, Continue<S>>,
{
    type Break = B;

    fn enter_mut(&mut self, node: &mut dyn Any) -> ControlFlow<B, Continue> {
        if let Some(node) = node.downcast_mut::<T>() {
            match (self.visit)(node, VisitEvent::Enter(self.states.last()))? {
                Continue::Children(state) => self.states.push(state),
                Continue::Skip => return ControlFlow::Continue(Continue::Skip),
            }
        }
        ControlFlow::Continue(Continue::Children(()))
    }

    fn leave_mut(&mut self, node: &mut dyn Any) -> ControlFlow<B> {
        if let Some(node) = node.downcast_mut::<T>() {
            (self.visit)(node, VisitEvent::Leave(self.states.pop().unwrap()))?;
        }
        ControlFlow::Continue(())
    }
}
