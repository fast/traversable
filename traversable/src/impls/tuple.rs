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

use crate::Traversable;
use crate::TraversableMut;
use crate::Visitor;
use crate::VisitorMut;

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
