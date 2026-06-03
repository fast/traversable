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

use crate::Folder;
use crate::Traversable;
use crate::TraversableFold;
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

fn traverse_fold_items<T, V, I, C>(items: I, folder: &mut V) -> ControlFlow<V::Break, C>
where
    T: TraversableFold,
    V: Folder,
    I: IntoIterator<Item = T>,
    C: FromIterator<T>,
{
    let mut folded = std::vec::Vec::new();
    for item in items {
        folded.push(item.traverse_fold(folder)?);
    }
    ControlFlow::Continue(folded.into_iter().collect())
}

fn traverse_fold_pairs<K, Value, V, I, C>(items: I, folder: &mut V) -> ControlFlow<V::Break, C>
where
    K: TraversableFold,
    Value: TraversableFold,
    V: Folder,
    I: IntoIterator<Item = (K, Value)>,
    C: FromIterator<(K, Value)>,
{
    let mut folded = std::vec::Vec::new();
    for (key, value) in items {
        let key = key.traverse_fold(folder)?;
        let value = value.traverse_fold(folder)?;
        folded.push((key, value));
    }
    ControlFlow::Continue(folded.into_iter().collect())
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

macro_rules! impl_fold_for_collection {
    ( $type:ty ; $($generics:tt)+ ) => {
        impl< $($generics)+ > TraversableFold for $type
        where
            $type: 'static + IntoIterator + FromIterator<<$type as IntoIterator>::Item>,
            <$type as IntoIterator>::Item: TraversableFold,
        {
            fn traverse_fold<V: Folder>(self, folder: &mut V) -> ControlFlow<V::Break, Self> {
                traverse_fold_items(self, folder)
            }
        }
    };
}

macro_rules! impl_fold_for_map {
    ( $type:ty ; $($generics:tt)+ ) => {
        impl< $($generics)+ > TraversableFold for $type
        where
            $type: 'static + IntoIterator<Item = (T, U)> + FromIterator<(T, U)>,
            T: TraversableFold,
            U: TraversableFold,
        {
            fn traverse_fold<V: Folder>(self, folder: &mut V) -> ControlFlow<V::Break, Self> {
                traverse_fold_pairs(self, folder)
            }
        }
    };
}

impl<T: TraversableFold, const N: usize> TraversableFold for [T; N] {
    fn traverse_fold<V: Folder>(self, folder: &mut V) -> ControlFlow<V::Break, Self> {
        let mut folded = std::vec::Vec::with_capacity(N);
        for item in self {
            folded.push(item.traverse_fold(folder)?);
        }
        let folded = match folded.try_into() {
            Ok(folded) => folded,
            Err(_) => unreachable!("folded array length must match the input array length"),
        };
        ControlFlow::Continue(folded)
    }
}

impl_fold_for_collection! { std::vec::Vec<T> ; T }
impl_fold_for_collection! { std::collections::BTreeSet<T> ; T }
impl_fold_for_collection! { std::collections::BinaryHeap<T> ; T }
impl_fold_for_collection! { std::collections::HashSet<T> ; T }
impl_fold_for_collection! { std::collections::LinkedList<T> ; T }
impl_fold_for_collection! { std::collections::VecDeque<T> ; T }
impl_fold_for_map! { std::collections::BTreeMap<T, U> ; T, U }
impl_fold_for_map! { std::collections::HashMap<T, U> ; T, U }

impl<T: TraversableFold> TraversableFold for Option<T> {
    fn traverse_fold<V: Folder>(self, folder: &mut V) -> ControlFlow<V::Break, Self> {
        match self {
            Some(item) => ControlFlow::Continue(Some(item.traverse_fold(folder)?)),
            None => ControlFlow::Continue(None),
        }
    }
}

impl<T, U> TraversableFold for Result<T, U>
where
    T: TraversableFold,
    U: 'static,
{
    fn traverse_fold<V: Folder>(self, folder: &mut V) -> ControlFlow<V::Break, Self> {
        match self {
            Ok(item) => ControlFlow::Continue(Ok(item.traverse_fold(folder)?)),
            Err(error) => ControlFlow::Continue(Err(error)),
        }
    }
}

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

impl<T: TraversableFold> TraversableFold for Box<T> {
    fn traverse_fold<V: Folder>(self, folder: &mut V) -> ControlFlow<V::Break, Self> {
        ControlFlow::Continue(Box::new((*self).traverse_fold(folder)?))
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

impl<T> TraversableFold for Mutex<T>
where
    T: TraversableFold,
{
    fn traverse_fold<V: Folder>(self, folder: &mut V) -> ControlFlow<V::Break, Self> {
        let item = self.into_inner().unwrap();
        let item = item.traverse_fold(folder)?;
        ControlFlow::Continue(Mutex::new(item))
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

impl<T> TraversableFold for RwLock<T>
where
    T: TraversableFold,
{
    fn traverse_fold<V: Folder>(self, folder: &mut V) -> ControlFlow<V::Break, Self> {
        let item = self.into_inner().unwrap();
        let item = item.traverse_fold(folder)?;
        ControlFlow::Continue(RwLock::new(item))
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

impl<T> TraversableFold for Cell<T>
where
    T: TraversableFold,
{
    fn traverse_fold<V: Folder>(self, folder: &mut V) -> ControlFlow<V::Break, Self> {
        let item = self.into_inner().traverse_fold(folder)?;
        ControlFlow::Continue(Cell::new(item))
    }
}
