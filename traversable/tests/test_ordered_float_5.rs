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

#![cfg(all(feature = "derive", feature = "ordered-float-5"))]

use std::any::Any;
use std::ops::ControlFlow;

use ordered_float_5::OrderedFloat;
use traversable::Continue;
use traversable::Traversable as _;
use traversable::TraversableMut as _;
use traversable::Visitor;
use traversable::VisitorMut;
use traversable_derive::Traversable;
use traversable_derive::TraversableMut;

#[test]
fn test_ordered_float() {
    #[derive(Debug, Clone, PartialEq, Eq, Traversable, TraversableMut)]
    pub enum Literal {
        Null,
        Float(OrderedFloat<f64>),
    }

    struct FloatExpr(bool);

    impl Visitor for FloatExpr {
        type Break = ();

        fn enter(&mut self, this: &dyn Any) -> ControlFlow<Self::Break, Continue> {
            if let Some(Literal::Float(_)) = this.downcast_ref::<Literal>() {
                self.0 = true;
            }
            ControlFlow::Continue(Continue::Children(()))
        }
    }

    impl VisitorMut for FloatExpr {
        type Break = ();

        fn enter_mut(&mut self, this: &mut dyn Any) -> ControlFlow<Self::Break, Continue> {
            if let Some(Literal::Float(_)) = this.downcast_ref::<Literal>() {
                self.0 = true;
            }
            ControlFlow::Continue(Continue::Children(()))
        }
    }

    assert!({
        let mut visitor = FloatExpr(false);
        let _ = Literal::Null.traverse(&mut visitor);
        !visitor.0
    });

    assert!({
        let mut visitor = FloatExpr(false);
        let _ = Literal::Null.traverse_mut(&mut visitor);
        !visitor.0
    });

    assert!({
        let mut visitor = FloatExpr(false);
        let _ = Literal::Float(OrderedFloat(0.0)).traverse(&mut visitor);
        visitor.0
    });

    assert!({
        let mut visitor = FloatExpr(false);
        let _ = Literal::Float(OrderedFloat(0.0)).traverse_mut(&mut visitor);
        visitor.0
    });
}
