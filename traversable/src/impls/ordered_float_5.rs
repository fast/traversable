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

use ordered_float_5::OrderedFloat;

use crate::Folder;
use crate::Traversable;
use crate::TraversableFold;
use crate::TraversableMut;
use crate::Visitor;
use crate::VisitorMut;

impl<T: 'static> Traversable for OrderedFloat<T> {
    fn traverse<V: Visitor>(&self, visitor: &mut V) -> ControlFlow<V::Break> {
        visitor.enter(self)?;
        visitor.leave(self)?;
        ControlFlow::Continue(())
    }
}

impl<T: 'static> TraversableMut for OrderedFloat<T> {
    fn traverse_mut<V: VisitorMut>(&mut self, visitor: &mut V) -> ControlFlow<V::Break> {
        visitor.enter_mut(self)?;
        visitor.leave_mut(self)?;
        ControlFlow::Continue(())
    }
}

impl<T: 'static> TraversableFold for OrderedFloat<T> {
    fn traverse_fold<V: Folder>(self, folder: &mut V) -> ControlFlow<V::Break, Self> {
        let this = folder.enter(self)?;
        let this = folder.leave(this)?;
        ControlFlow::Continue(this)
    }
}
