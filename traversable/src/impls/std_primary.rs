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
use std::string::String;

use crate::Folder;
use crate::Traversable;
use crate::TraversableFold;
use crate::TraversableMut;
use crate::Visitor;
use crate::VisitorMut;

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
