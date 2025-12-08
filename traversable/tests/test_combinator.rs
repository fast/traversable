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
use std::cell::RefCell;
use std::ops::ControlFlow;
use std::rc::Rc;

use traversable::Traversable;
use traversable::TraversableMut;
use traversable::Visitor;
use traversable::VisitorMut;
use traversable::combinator::VisitorExt;
use traversable::combinator::VisitorMutExt;

#[derive(Traversable, TraversableMut)]
struct Data {
    val: i32,
    child: Option<Box<Data>>,
}

#[derive(Clone)]
struct RecordVisitor {
    log: Rc<RefCell<Vec<String>>>,
    name: String,
    break_on: Option<i32>,
}

impl Visitor for RecordVisitor {
    type Break = i32;

    fn enter(&mut self, this: &dyn Any) -> ControlFlow<Self::Break> {
        if let Some(d) = this.downcast_ref::<Data>() {
            self.log
                .borrow_mut()
                .push(format!("{}: enter {}", self.name, d.val));
            if Some(d.val) == self.break_on {
                return ControlFlow::Break(d.val);
            }
        }
        ControlFlow::Continue(())
    }

    fn leave(&mut self, this: &dyn Any) -> ControlFlow<Self::Break> {
        if let Some(d) = this.downcast_ref::<Data>() {
            self.log
                .borrow_mut()
                .push(format!("{}: leave {}", self.name, d.val));
        }
        ControlFlow::Continue(())
    }
}

#[test]
fn test_visitor_or() {
    let data = Data {
        val: 1,
        child: Some(Box::new(Data {
            val: 2,
            child: None,
        })),
    };

    let log = Rc::new(RefCell::new(Vec::new()));
    let v1 = RecordVisitor {
        log: log.clone(),
        name: "v1".to_string(),
        break_on: None,
    };
    let v2 = RecordVisitor {
        log: log.clone(),
        name: "v2".to_string(),
        break_on: None,
    };

    let mut combined = v1.or(v2);
    let result = data.traverse(&mut combined);
    assert!(result.is_continue());

    let expected = vec![
        "v1: enter 1",
        "v2: enter 1",
        "v1: enter 2",
        "v2: enter 2",
        "v1: leave 2",
        "v2: leave 2",
        "v1: leave 1",
        "v2: leave 1",
    ];
    assert_eq!(*log.borrow(), expected);
}

#[test]
fn test_visitor_or_break_v1() {
    let data = Data {
        val: 1,
        child: Some(Box::new(Data {
            val: 2,
            child: None,
        })),
    };

    let log = Rc::new(RefCell::new(Vec::new()));
    // v1 breaks on 2
    let v1 = RecordVisitor {
        log: log.clone(),
        name: "v1".to_string(),
        break_on: Some(2),
    };
    let v2 = RecordVisitor {
        log: log.clone(),
        name: "v2".to_string(),
        break_on: None,
    };

    let mut combined = v1.or(v2);
    let result = data.traverse(&mut combined);
    assert_eq!(result, ControlFlow::Break(2));

    let expected = vec![
        "v1: enter 1",
        "v2: enter 1",
        "v1: enter 2", // v1 breaks here, v2 not called for 2, and traversal stops
    ];
    assert_eq!(*log.borrow(), expected);
}

#[test]
fn test_visitor_or_break_v2() {
    let data = Data {
        val: 1,
        child: Some(Box::new(Data {
            val: 2,
            child: None,
        })),
    };

    let log = Rc::new(RefCell::new(Vec::new()));
    // v2 breaks on 2
    let v1 = RecordVisitor {
        log: log.clone(),
        name: "v1".to_string(),
        break_on: None,
    };
    let v2 = RecordVisitor {
        log: log.clone(),
        name: "v2".to_string(),
        break_on: Some(2),
    };

    let mut combined = v1.or(v2);
    let result = data.traverse(&mut combined);
    assert_eq!(result, ControlFlow::Break(2));

    let expected = vec![
        "v1: enter 1",
        "v2: enter 1",
        "v1: enter 2",
        "v2: enter 2", // v2 breaks here
    ];
    assert_eq!(*log.borrow(), expected);
}

struct MutVisitor {
    val_multiplier: i32,
}

impl VisitorMut for MutVisitor {
    type Break = ();
    fn enter_mut(&mut self, this: &mut dyn Any) -> ControlFlow<()> {
        if let Some(d) = this.downcast_mut::<Data>() {
            d.val *= self.val_multiplier;
        }
        ControlFlow::Continue(())
    }
}

#[test]
fn test_visitor_mut_or() {
    let mut data = Data {
        val: 1,
        child: Some(Box::new(Data {
            val: 2,
            child: None,
        })),
    };

    let v1 = MutVisitor { val_multiplier: 2 };
    let v2 = MutVisitor { val_multiplier: 3 };

    let mut combined = v1.or(v2);
    let result = data.traverse_mut(&mut combined);
    assert!(result.is_continue());

    // 1 * 2 * 3 = 6
    assert_eq!(data.val, 6);
    // 2 * 2 * 3 = 12
    assert_eq!(data.child.unwrap().val, 12);
}
