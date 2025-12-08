//! Combinators for Visitors.

use crate::{Visitor, VisitorMut};

///
pub trait VisitorExt: Visitor {
    ///
    fn or<V>(self, other: V) -> OrVisitor<Self, V>
    where
        Self: Sized,
        V: Visitor<Break = Self::Break>,
    {
        OrVisitor {
            visitor1: self,
            visitor2: other,
        }
    }
}

impl<V: Visitor> VisitorExt for V {}

///
pub trait VisitorMutExt: VisitorMut {
    ///
    fn or<V>(self, other: V) -> OrVisitor<Self, V>
    where
        Self: Sized,
        V: VisitorMut<Break = Self::Break>,
    {
        OrVisitor {
            visitor1: self,
            visitor2: other,
        }
    }
}

impl<V: VisitorMut> VisitorMutExt for V {}

///
pub struct OrVisitor<V1, V2> {
    visitor1: V1,
    visitor2: V2,
}

impl<V1, V2> Visitor for OrVisitor<V1, V2>
where
    V1: Visitor,
    V2: Visitor<Break = V1::Break>,
{
    type Break = V1::Break;

    fn enter(&mut self, this: &dyn core::any::Any) -> core::ops::ControlFlow<Self::Break> {
        self.visitor1.enter(this)?;
        self.visitor2.enter(this)
    }

    fn leave(&mut self, this: &dyn core::any::Any) -> core::ops::ControlFlow<Self::Break> {
        self.visitor1.leave(this)?;
        self.visitor2.leave(this)
    }
}

impl<V1, V2> VisitorMut for OrVisitor<V1, V2>
where
    V1: VisitorMut,
    V2: VisitorMut<Break = V1::Break>,
{
    type Break = V1::Break;

    fn enter_mut(&mut self, this: &mut dyn core::any::Any) -> core::ops::ControlFlow<Self::Break> {
        self.visitor1.enter_mut(this)?;
        self.visitor2.enter_mut(this)
    }

    fn leave_mut(&mut self, this: &mut dyn core::any::Any) -> core::ops::ControlFlow<Self::Break> {
        self.visitor1.leave_mut(this)?;
        self.visitor2.leave_mut(this)
    }
}
