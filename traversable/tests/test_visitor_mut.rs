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

#![cfg(all(feature = "std", feature = "derive"))]

use std::any::Any;
use std::ops::ControlFlow;

use traversable::Continue;
use traversable::TraversableMut;
use traversable::VisitorMut;

#[derive(TraversableMut)]
struct Chain {
    next: Option<Box<Chain>>,
}

impl Chain {
    fn depth(&self) -> usize {
        if let Some(child) = &self.next {
            1 + child.depth()
        } else {
            0
        }
    }
}

struct ChainCutter {
    cut_at_depth: usize,
}

impl VisitorMut for ChainCutter {
    type Break = ();

    fn enter_mut(&mut self, this: &mut dyn Any) -> ControlFlow<Self::Break, Continue> {
        if let Some(item) = this.downcast_mut::<Chain>() {
            if self.cut_at_depth == 0 {
                item.next = None;
            } else {
                self.cut_at_depth -= 1;
            }
        }

        ControlFlow::Continue(Continue::Children(()))
    }
}

#[test]
fn test() {
    let mut chain = Chain {
        next: Some(Box::new(Chain {
            next: Some(Box::new(Chain { next: None })),
        })),
    };
    assert_eq!(chain.depth(), 2);

    let mut cutter = ChainCutter { cut_at_depth: 1 };
    let result = chain.traverse_mut(&mut cutter);
    assert!(result.is_continue());
    assert_eq!(chain.depth(), 1);
}
