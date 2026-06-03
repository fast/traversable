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

use crate::Folder;
use crate::Traversable;
use crate::TraversableFold;
use crate::TraversableMut;
use crate::Visitor;
use crate::VisitorMut;

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
