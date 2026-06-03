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

use core::ops::ControlFlow;
use std::boxed::Box;
use std::cell::Cell;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::RwLock;

use crate::Traversable;
use crate::TraversableMut;
use crate::Visitor;
use crate::VisitorMut;

trait DerefAndTraverse {
    fn deref_and_traverse<V: Visitor>(self, visitor: &mut V) -> ControlFlow<V::Break>;
}

trait DerefAndTraverseMut {
    fn deref_and_traverse_mut<V: VisitorMut>(self, visitor: &mut V) -> ControlFlow<V::Break>;
}

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

impl<TK: Traversable, TV: Traversable> DerefAndTraverse for (&TK, &TV) {
    fn deref_and_traverse<V: Visitor>(self, visitor: &mut V) -> ControlFlow<V::Break> {
        self.0.traverse(visitor)?;
        self.1.traverse(visitor)?;
        ControlFlow::Continue(())
    }
}

impl<TK, TV: TraversableMut> DerefAndTraverseMut for (TK, &mut TV) {
    fn deref_and_traverse_mut<V: VisitorMut>(self, visitor: &mut V) -> ControlFlow<V::Break> {
        self.1.traverse_mut(visitor)
    }
}

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

impl<T> TraversableMut for Mutex<T>
where
    T: TraversableMut,
{
    fn traverse_mut<V: VisitorMut>(&mut self, visitor: &mut V) -> ControlFlow<V::Break> {
        let lock = self.get_mut().unwrap();
        lock.traverse_mut(visitor)
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

impl<T> TraversableMut for RwLock<T>
where
    T: TraversableMut,
{
    fn traverse_mut<V: VisitorMut>(&mut self, visitor: &mut V) -> ControlFlow<V::Break> {
        let lock = self.get_mut().unwrap();
        lock.traverse_mut(visitor)
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
