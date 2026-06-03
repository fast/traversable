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

use core::any::Any;
use core::any::TypeId;
use core::ops::ControlFlow;

use traversable::Folder;
use traversable::TraversableFold;
use traversable::function::folder_leave;

fn into_continue<T>(flow: ControlFlow<(), T>) -> T {
    match flow {
        ControlFlow::Continue(value) => value,
        ControlFlow::Break(()) => unreachable!(),
    }
}

#[derive(Debug, PartialEq, Eq, TraversableFold)]
enum Expr {
    Add(Box<Expr>, Box<Expr>),
    Literal(i32),
}

fn simplify(expr: Expr) -> Expr {
    match expr {
        Expr::Add(left, right) => match (*left, *right) {
            (Expr::Literal(0), expr) | (expr, Expr::Literal(0)) => expr,
            (left, right) => Expr::Add(Box::new(left), Box::new(right)),
        },
        expr => expr,
    }
}

#[test]
fn folder_leave_rewrites_bottom_up() {
    let expr = Expr::Add(
        Box::new(Expr::Add(
            Box::new(Expr::Literal(0)),
            Box::new(Expr::Literal(1)),
        )),
        Box::new(Expr::Literal(0)),
    );

    let mut folder = folder_leave::<Expr, (), _>(|expr| ControlFlow::Continue(simplify(expr)));
    let expr = into_continue(expr.traverse_fold(&mut folder));

    assert_eq!(expr, Expr::Literal(1));
}

#[derive(Debug, PartialEq, Eq, TraversableFold)]
struct Child {
    value: u64,
}

#[derive(TraversableFold)]
struct Pair {
    #[traverse(with = "fold_and_double")]
    folded: Child,
    #[traverse(skip)]
    skipped: Child,
}

fn fold_and_double<V: Folder>(child: Child, folder: &mut V) -> ControlFlow<V::Break, Child> {
    let mut child = child.traverse_fold(folder)?;
    child.value *= 2;
    ControlFlow::Continue(child)
}

#[test]
fn field_attributes_control_fold_behavior() {
    let pair = Pair {
        folded: Child { value: 1 },
        skipped: Child { value: 10 },
    };

    let mut folder = folder_leave::<Child, (), _>(|mut child| {
        child.value += 1;
        ControlFlow::Continue(child)
    });
    let pair = into_continue(pair.traverse_fold(&mut folder));

    assert_eq!(pair.folded, Child { value: 4 });
    assert_eq!(pair.skipped, Child { value: 10 });
}

#[test]
fn std_containers_fold_owned_items() {
    let mut folder = folder_leave::<Child, (), _>(|mut child| {
        child.value += 1;
        ControlFlow::Continue(child)
    });

    let values = vec![Child { value: 1 }, Child { value: 2 }];
    let values = into_continue(values.traverse_fold(&mut folder));

    assert_eq!(values, vec![Child { value: 2 }, Child { value: 3 }]);

    let value = Some(Child { value: 4 });
    let value = into_continue(value.traverse_fold(&mut folder));

    assert_eq!(value, Some(Child { value: 5 }));

    let value = Ok::<_, Child>(Child { value: 6 });
    let value = into_continue(value.traverse_fold(&mut folder));

    assert_eq!(value, Ok(Child { value: 7 }));
}

#[derive(TraversableFold)]
struct Parent {
    child: Child,
}

#[derive(TraversableFold)]
#[traverse(skip_self)]
struct SkipSelfParent {
    child: Child,
}

#[derive(TraversableFold)]
#[traverse(skip_children)]
#[allow(dead_code)]
struct SkipChildrenParent {
    child: Child,
}

#[derive(TraversableFold)]
#[traverse(skip_self, skip_children)]
#[allow(dead_code)]
struct SkipSelfAndChildrenParent {
    child: Child,
}

#[derive(Default)]
struct Counts {
    parent_enter: usize,
    parent_leave: usize,
    child_enter: usize,
    child_leave: usize,
    skip_self_parent_enter: usize,
    skip_self_parent_leave: usize,
    skip_children_parent_enter: usize,
    skip_children_parent_leave: usize,
    skip_self_and_children_parent_enter: usize,
    skip_self_and_children_parent_leave: usize,
}

impl Folder for Counts {
    type Break = ();

    fn enter<T: Any>(&mut self, this: T) -> ControlFlow<Self::Break, T> {
        if TypeId::of::<T>() == TypeId::of::<Parent>() {
            self.parent_enter += 1;
        } else if TypeId::of::<T>() == TypeId::of::<Child>() {
            self.child_enter += 1;
        } else if TypeId::of::<T>() == TypeId::of::<SkipSelfParent>() {
            self.skip_self_parent_enter += 1;
        } else if TypeId::of::<T>() == TypeId::of::<SkipChildrenParent>() {
            self.skip_children_parent_enter += 1;
        } else if TypeId::of::<T>() == TypeId::of::<SkipSelfAndChildrenParent>() {
            self.skip_self_and_children_parent_enter += 1;
        }

        ControlFlow::Continue(this)
    }

    fn leave<T: Any>(&mut self, this: T) -> ControlFlow<Self::Break, T> {
        if TypeId::of::<T>() == TypeId::of::<Parent>() {
            self.parent_leave += 1;
        } else if TypeId::of::<T>() == TypeId::of::<Child>() {
            self.child_leave += 1;
        } else if TypeId::of::<T>() == TypeId::of::<SkipSelfParent>() {
            self.skip_self_parent_leave += 1;
        } else if TypeId::of::<T>() == TypeId::of::<SkipChildrenParent>() {
            self.skip_children_parent_leave += 1;
        } else if TypeId::of::<T>() == TypeId::of::<SkipSelfAndChildrenParent>() {
            self.skip_self_and_children_parent_leave += 1;
        }

        ControlFlow::Continue(this)
    }
}

#[test]
fn type_level_attributes_control_fold_behavior() {
    let mut counts = Counts::default();
    let parent = Parent {
        child: Child { value: 1 },
    };

    let result = parent.traverse_fold(&mut counts);

    assert!(result.is_continue());
    assert_eq!(counts.parent_enter, 1);
    assert_eq!(counts.parent_leave, 1);
    assert_eq!(counts.child_enter, 1);
    assert_eq!(counts.child_leave, 1);

    let mut counts = Counts::default();
    let skip_self = SkipSelfParent {
        child: Child { value: 1 },
    };

    let result = skip_self.traverse_fold(&mut counts);

    assert!(result.is_continue());
    assert_eq!(counts.skip_self_parent_enter, 0);
    assert_eq!(counts.skip_self_parent_leave, 0);
    assert_eq!(counts.child_enter, 1);
    assert_eq!(counts.child_leave, 1);

    let mut counts = Counts::default();
    let skip_children = SkipChildrenParent {
        child: Child { value: 1 },
    };

    let result = skip_children.traverse_fold(&mut counts);

    assert!(result.is_continue());
    assert_eq!(counts.skip_children_parent_enter, 1);
    assert_eq!(counts.skip_children_parent_leave, 1);
    assert_eq!(counts.child_enter, 0);
    assert_eq!(counts.child_leave, 0);

    let mut counts = Counts::default();
    let skip_self_and_children = SkipSelfAndChildrenParent {
        child: Child { value: 1 },
    };

    let result = skip_self_and_children.traverse_fold(&mut counts);

    assert!(result.is_continue());
    assert_eq!(counts.skip_self_and_children_parent_enter, 0);
    assert_eq!(counts.skip_self_and_children_parent_leave, 0);
    assert_eq!(counts.child_enter, 0);
    assert_eq!(counts.child_leave, 0);
}
