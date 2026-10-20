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

#![cfg(all(feature = "derive", feature = "std"))]

use core::ops::ControlFlow;

use traversable::Continue;
use traversable::Traversable;
use traversable::TraversableMut;
use traversable::Visitor;
use traversable::VisitorMut;
use traversable::combinator::VisitorExt;
use traversable::combinator::VisitorMutExt;
use traversable::function::VisitEvent;
use traversable::function::visitor;
use traversable::function::visitor_enter_scoped;
use traversable::function::visitor_enter_scoped_mut;
use traversable::function::visitor_events;
use traversable::function::visitor_events_mut;
use traversable::function::visitor_mut;

#[derive(Traversable, TraversableMut)]
enum Node {
    Branch(Vec<Node>),
    Leaf(i32),
    Pruned(Trap),
}

struct Trap;

impl Traversable for Trap {
    fn traverse<V: Visitor>(&self, _: &mut V) -> ControlFlow<V::Break> {
        panic!("skipped subtree was traversed")
    }
}

impl TraversableMut for Trap {
    fn traverse_mut<V: VisitorMut>(&mut self, _: &mut V) -> ControlFlow<V::Break> {
        panic!("skipped subtree was traversed")
    }
}

#[test]
fn skip_subtree() {
    let mut nodes = vec![Node::Pruned(Trap), Node::Leaf(1)];
    let mut leaves = 0;
    assert!(
        nodes
            .traverse(&mut visitor(
                |node: &Node| ControlFlow::<(), _>::Continue(match node {
                    Node::Pruned(_) => Continue::Skip,
                    _ => Continue::Children(()),
                }),
                |node: &Node| {
                    assert!(matches!(node, Node::Leaf(_)));
                    leaves += 1;
                    ControlFlow::<(), _>::Continue(())
                },
            ))
            .is_continue()
    );
    assert_eq!(leaves, 1);

    assert!(
        nodes
            .traverse_mut(&mut visitor_mut(
                |node: &mut Node| {
                    match node {
                        Node::Leaf(value) => *value += 1,
                        _ => {
                            // A replacement must not be traversed after Skip either.
                            *node = Node::Branch(vec![Node::Pruned(Trap)]);
                            return ControlFlow::<(), _>::Continue(Continue::Skip);
                        }
                    }
                    ControlFlow::Continue(Continue::Children(()))
                },
                |node: &mut Node| {
                    assert!(matches!(node, Node::Leaf(_)));
                    ControlFlow::<(), _>::Continue(())
                },
            ))
            .is_continue()
    );
    assert!(matches!(nodes[1], Node::Leaf(2)));
}

#[test]
fn scoped_state_with_chained_skip() {
    let mut nodes = vec![
        Node::Branch(vec![Node::Pruned(Trap), Node::Leaf(0)]),
        Node::Leaf(0),
    ];
    let mut depths = Vec::new();
    let state = visitor_enter_scoped(|node: &Node, parent: Option<&i32>| {
        let depth = parent.copied().unwrap_or(0) + 1;
        if let Node::Leaf(_) = node {
            depths.push(depth);
        }
        ControlFlow::<(), _>::Continue(Continue::Children(depth))
    });
    let skip = visitor_events(|node: &Node, event| {
        ControlFlow::<(), _>::Continue(match event {
            VisitEvent::Enter(_) if matches!(node, Node::Pruned(_)) => Continue::Skip,
            VisitEvent::Enter(_) => Continue::Children(()),
            VisitEvent::Leave(()) => {
                assert!(!matches!(node, Node::Pruned(_)));
                Continue::Skip
            }
        })
    });
    assert!(nodes.traverse(&mut state.chain(skip)).is_continue());
    assert_eq!(depths, [2, 1]);

    // Reversing the chain also covers a skip by the first visitor.
    let skip = visitor_enter_scoped_mut(|node: &mut Node, _| {
        ControlFlow::<i32, _>::Continue(match node {
            Node::Pruned(_) => Continue::Skip,
            _ => Continue::Children(()),
        })
    });
    let state = visitor_events_mut(|node: &mut Node, event| {
        assert!(!matches!(node, Node::Pruned(_)));
        ControlFlow::<i32, _>::Continue(match event {
            VisitEvent::Enter(parent) => Continue::Children(parent.copied().unwrap_or(0) + 1),
            VisitEvent::Leave(depth) => {
                if let Node::Leaf(value) = node {
                    *value = depth;
                    if depth == 1 {
                        return ControlFlow::Break(depth);
                    }
                }
                Continue::Skip
            }
        })
    });
    assert_eq!(
        nodes.traverse_mut(&mut skip.chain(state)),
        ControlFlow::Break(1)
    );
    let Node::Branch(children) = &nodes[0] else {
        panic!()
    };
    assert!(matches!(children[1], Node::Leaf(2)));
    assert!(matches!(nodes[1], Node::Leaf(1)));
}
