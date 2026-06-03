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

use traversable::Traversable;
use traversable::TraversableMut;
use traversable::Visitor;
use traversable::VisitorMut;

#[derive(Traversable, TraversableMut)]
struct Child {
    value: u64,
}

#[derive(Traversable, TraversableMut)]
struct Parent {
    child: Child,
}

#[derive(Traversable, TraversableMut)]
#[traverse(skip_self)]
struct SkipSelfParent {
    child: Child,
}

#[derive(Traversable, TraversableMut)]
#[traverse(skip_children)]
#[allow(dead_code)]
struct SkipChildrenParent {
    child: Child,
}

#[derive(Traversable, TraversableMut)]
#[traverse(skip_self, skip_children)]
#[allow(dead_code)]
struct SkipSelfAndChildrenParent {
    child: Child,
}

#[derive(Traversable)]
#[traverse(skip_children)]
#[allow(dead_code)]
enum SkipChildrenEnum {
    Child(Child),
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
    skip_children_enum_enter: usize,
    skip_children_enum_leave: usize,
}

impl Visitor for Counts {
    type Break = ();

    fn enter(&mut self, this: &dyn Any) -> ControlFlow<Self::Break> {
        if this.is::<Parent>() {
            self.parent_enter += 1;
        } else if this.is::<Child>() {
            self.child_enter += 1;
        } else if this.is::<SkipSelfParent>() {
            self.skip_self_parent_enter += 1;
        } else if this.is::<SkipChildrenParent>() {
            self.skip_children_parent_enter += 1;
        } else if this.is::<SkipSelfAndChildrenParent>() {
            self.skip_self_and_children_parent_enter += 1;
        } else if this.is::<SkipChildrenEnum>() {
            self.skip_children_enum_enter += 1;
        }

        ControlFlow::Continue(())
    }

    fn leave(&mut self, this: &dyn Any) -> ControlFlow<Self::Break> {
        if this.is::<Parent>() {
            self.parent_leave += 1;
        } else if this.is::<Child>() {
            self.child_leave += 1;
        } else if this.is::<SkipSelfParent>() {
            self.skip_self_parent_leave += 1;
        } else if this.is::<SkipChildrenParent>() {
            self.skip_children_parent_leave += 1;
        } else if this.is::<SkipSelfAndChildrenParent>() {
            self.skip_self_and_children_parent_leave += 1;
        } else if this.is::<SkipChildrenEnum>() {
            self.skip_children_enum_leave += 1;
        }

        ControlFlow::Continue(())
    }
}

impl VisitorMut for Counts {
    type Break = ();

    fn enter_mut(&mut self, this: &mut dyn Any) -> ControlFlow<Self::Break> {
        if this.is::<Parent>() {
            self.parent_enter += 1;
        } else if this.is::<Child>() {
            self.child_enter += 1;
        } else if this.is::<SkipSelfParent>() {
            self.skip_self_parent_enter += 1;
        } else if this.is::<SkipChildrenParent>() {
            self.skip_children_parent_enter += 1;
        } else if this.is::<SkipSelfAndChildrenParent>() {
            self.skip_self_and_children_parent_enter += 1;
        }

        ControlFlow::Continue(())
    }

    fn leave_mut(&mut self, this: &mut dyn Any) -> ControlFlow<Self::Break> {
        if this.is::<Parent>() {
            self.parent_leave += 1;
        } else if this.is::<Child>() {
            self.child_leave += 1;
        } else if this.is::<SkipSelfParent>() {
            self.skip_self_parent_leave += 1;
        } else if this.is::<SkipChildrenParent>() {
            self.skip_children_parent_leave += 1;
        } else if this.is::<SkipSelfAndChildrenParent>() {
            self.skip_self_and_children_parent_leave += 1;
        }

        ControlFlow::Continue(())
    }
}

#[test]
fn traversable_visits_self_and_children_by_default() {
    let parent = Parent {
        child: Child { value: 1 },
    };
    let mut counts = Counts::default();

    let result = parent.traverse(&mut counts);

    assert!(result.is_continue());
    assert_eq!(counts.parent_enter, 1);
    assert_eq!(counts.parent_leave, 1);
    assert_eq!(counts.child_enter, 1);
    assert_eq!(counts.child_leave, 1);
}

#[test]
fn skip_self_visits_children_only() {
    let parent = SkipSelfParent {
        child: Child { value: 1 },
    };
    let mut counts = Counts::default();

    let result = parent.traverse(&mut counts);

    assert!(result.is_continue());
    assert_eq!(counts.skip_self_parent_enter, 0);
    assert_eq!(counts.skip_self_parent_leave, 0);
    assert_eq!(counts.child_enter, 1);
    assert_eq!(counts.child_leave, 1);
}

#[test]
fn skip_children_visits_self_only() {
    let parent = SkipChildrenParent {
        child: Child { value: 1 },
    };
    let mut counts = Counts::default();

    let result = parent.traverse(&mut counts);

    assert!(result.is_continue());
    assert_eq!(counts.skip_children_parent_enter, 1);
    assert_eq!(counts.skip_children_parent_leave, 1);
    assert_eq!(counts.child_enter, 0);
    assert_eq!(counts.child_leave, 0);
}

#[test]
fn skip_self_and_children_visits_nothing() {
    let parent = SkipSelfAndChildrenParent {
        child: Child { value: 1 },
    };
    let mut counts = Counts::default();

    let result = parent.traverse(&mut counts);

    assert!(result.is_continue());
    assert_eq!(counts.skip_self_and_children_parent_enter, 0);
    assert_eq!(counts.skip_self_and_children_parent_leave, 0);
    assert_eq!(counts.child_enter, 0);
    assert_eq!(counts.child_leave, 0);
}

#[test]
fn skip_children_visits_enum_self_only() {
    let item = SkipChildrenEnum::Child(Child { value: 1 });
    let mut counts = Counts::default();

    let result = item.traverse(&mut counts);

    assert!(result.is_continue());
    assert_eq!(counts.skip_children_enum_enter, 1);
    assert_eq!(counts.skip_children_enum_leave, 1);
    assert_eq!(counts.child_enter, 0);
    assert_eq!(counts.child_leave, 0);
}

#[test]
fn traversable_mut_honors_type_level_attributes() {
    let mut skip_self = SkipSelfParent {
        child: Child { value: 1 },
    };
    let mut counts = Counts::default();

    let result = skip_self.traverse_mut(&mut counts);

    assert!(result.is_continue());
    assert_eq!(counts.skip_self_parent_enter, 0);
    assert_eq!(counts.skip_self_parent_leave, 0);
    assert_eq!(counts.child_enter, 1);
    assert_eq!(counts.child_leave, 1);

    let mut skip_children = SkipChildrenParent {
        child: Child { value: 1 },
    };
    let mut counts = Counts::default();

    let result = skip_children.traverse_mut(&mut counts);

    assert!(result.is_continue());
    assert_eq!(counts.skip_children_parent_enter, 1);
    assert_eq!(counts.skip_children_parent_leave, 1);
    assert_eq!(counts.child_enter, 0);
    assert_eq!(counts.child_leave, 0);
}
