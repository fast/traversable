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

        impl TraversableFold for $type {
            #[inline]
            fn traverse_fold<V: Folder>(self, _folder: &mut V) -> ControlFlow<V::Break, Self> {
                ControlFlow::Continue(self)
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

        impl TraversableFold for $type {
            fn traverse_fold<V: Folder>(self, folder: &mut V) -> ControlFlow<V::Break, Self> {
                let this = folder.enter(self)?;
                let this = folder.leave(this)?;
                ControlFlow::Continue(this)
            }
        }
    };
}

#[cfg(feature = "ordered-float-5")]
mod ordered_float_5;
#[cfg(feature = "stacksafe-1")]
mod stacksafe_1;
#[cfg(feature = "std")]
mod std_container;
#[cfg(feature = "std")]
mod std_primary;
mod trivial;
mod tuple;
