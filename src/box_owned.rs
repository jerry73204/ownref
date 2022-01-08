use crate::{arc_owned::ArcOwned, arc_ref::ArcRef, box_ref::BoxRef, marker::*};
use std::{
    cmp, fmt,
    fmt::{Debug, Display},
    marker::PhantomData,
    ops::{Deref, DerefMut},
    ptr,
};

pub type BoxOwnedC<'a, O, I = O> = BoxOwned<'a, O, I, ByContent>;
pub type BoxOwnedA<'a, O, I = O> = BoxOwned<'a, O, I, ByAddress>;

pub struct BoxOwned<'a, O, I, E>
where
    O: ?Sized,
    E: EqKind,
{
    // inner goes before owner so that inner drops before owner
    pub(crate) _phantom: PhantomData<(&'a I, E)>,
    pub(crate) inner: I,
    pub(crate) owner: Box<O>,
}

impl<'a, O, E> BoxOwned<'a, O, &'a mut O, E>
where
    E: EqKind,
{
    pub fn new(owner: O) -> Self {
        Box::new(owner).into()
    }

    pub fn from_box(owner: Box<O>) -> Self {
        owner.into()
    }
}

impl<'a, O, I, E> BoxOwned<'a, O, I, E>
where
    E: EqKind,
{
    pub fn into_box(from: BoxOwned<'a, O, I, E>) -> Box<O> {
        let Self { owner, inner, .. } = from;
        drop(inner);
        owner
    }

    pub fn into_owner(from: BoxOwned<'a, O, I, E>) -> O {
        let Self { owner, inner, .. } = from;
        drop(inner);
        *owner
    }

    pub fn into_arc_owned(from: BoxOwned<'a, O, I, E>) -> ArcOwned<'a, O, I, E> {
        let Self { owner, inner, .. } = from;
        ArcOwned {
            owner: owner.into(),
            inner,
            _phantom: PhantomData,
        }
    }

    pub fn into_owner_ref(this: BoxOwned<'a, O, I, E>) -> BoxOwned<'a, O, &mut O, E> {
        let Self {
            mut owner, inner, ..
        } = this;
        drop(inner);

        unsafe {
            // re-borrow to obtain 'a lifetime
            let inner = &mut *(owner.as_mut() as *mut O);

            BoxOwned {
                inner,
                owner,
                _phantom: PhantomData,
            }
        }
    }

    pub fn owner(this: &'a BoxOwned<'a, O, I, E>) -> &'a O {
        &this.owner
    }

    pub fn map<T, F>(self, f: F) -> BoxOwned<'a, O, T, E>
    where
        F: FnOnce(I) -> T,
    {
        let Self { owner, inner, .. } = self;

        BoxOwned {
            owner,
            inner: f(inner),
            _phantom: PhantomData,
        }
    }

    pub fn try_map<Ok, Err, F>(self, f: F) -> Result<BoxOwned<'a, O, Ok, E>, Err>
    where
        F: FnOnce(I) -> Result<Ok, Err>,
    {
        let Self { owner, inner, .. } = self;

        Ok(BoxOwned {
            owner,
            inner: f(inner)?,
            _phantom: PhantomData,
        })
    }

    pub fn filter_map<T, F>(self, f: F) -> Option<BoxOwned<'a, O, T, E>>
    where
        F: FnOnce(I) -> Option<T>,
    {
        let Self { owner, inner, .. } = self;

        Some(BoxOwned {
            owner,
            inner: f(inner)?,
            _phantom: PhantomData,
        })
    }
}

impl<'a, O, I, E> BoxOwned<'a, O, &'a mut I, E>
where
    E: EqKind,
{
    pub fn into_box_ref(self) -> BoxRef<'a, O, I, E> {
        let Self { owner, inner, .. } = self;

        BoxRef {
            owner,
            inner,
            _phantom: PhantomData,
        }
    }

    pub fn into_arc_ref(self) -> ArcRef<'a, O, I, E> {
        let Self { owner, inner, .. } = self;

        ArcRef {
            owner: owner.into(),
            inner,
            _phantom: PhantomData,
        }
    }
}

impl<'a, O, I, E> BoxOwned<'a, O, &'a I, E>
where
    E: EqKind,
{
    pub fn into_arc_ref(self) -> ArcRef<'a, O, I, E> {
        let Self { owner, inner, .. } = self;

        ArcRef {
            owner: owner.into(),
            inner,
            _phantom: PhantomData,
        }
    }
}

impl<'a, O, I, E> BoxOwned<'a, O, Option<I>, E>
where
    E: EqKind,
{
    pub fn transpose(self) -> Option<BoxOwned<'a, O, I, E>> {
        let Self { owner, inner, .. } = self;
        Some(BoxOwned {
            owner,
            inner: inner?,
            _phantom: PhantomData,
        })
    }
}

impl<'a, O, Ok, Err, E> BoxOwned<'a, O, Result<Ok, Err>, E>
where
    E: EqKind,
{
    pub fn transpose(self) -> Result<BoxOwned<'a, O, Ok, E>, Err> {
        let Self { owner, inner, .. } = self;
        Ok(BoxOwned {
            owner,
            inner: inner?,
            _phantom: PhantomData,
        })
    }
}

impl<'a, O, I, E> Debug for BoxOwned<'a, O, I, E>
where
    I: Debug,
    E: EqKind,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        self.inner.fmt(f)
    }
}

impl<'a, O, I, E> Display for BoxOwned<'a, O, I, E>
where
    I: Display,
    E: EqKind,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        self.inner.fmt(f)
    }
}

impl<'a, O, I> PartialEq<Self> for BoxOwned<'a, O, I, ByContent>
where
    I: PartialEq<I>,
{
    fn eq(&self, other: &Self) -> bool {
        self.inner.eq(&other.inner)
    }
}

impl<'a, O, I> Eq for BoxOwned<'a, O, I, ByContent> where I: Eq {}

impl<'a, O, I> PartialOrd<Self> for BoxOwned<'a, O, I, ByContent>
where
    I: PartialOrd<I>,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        self.inner.partial_cmp(&other.inner)
    }
}

impl<'a, O, I> Ord for BoxOwned<'a, O, I, ByContent>
where
    I: Ord,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.inner.cmp(&other.inner)
    }
}

impl<'a, O, I> PartialEq<Self> for BoxOwned<'a, O, &'a mut I, ByAddress> {
    fn eq(&self, other: &Self) -> bool {
        ptr::eq(self.inner as *const I, other.inner as *const I)
    }
}

impl<'a, O, I> Eq for BoxOwned<'a, O, &'a mut I, ByAddress> {}

impl<'a, O, I> PartialOrd<Self> for BoxOwned<'a, O, &'a mut I, ByAddress> {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        (self.inner as *const I).partial_cmp(&(other.inner as *const I))
    }
}

impl<'a, O, I> Ord for BoxOwned<'a, O, &'a mut I, ByAddress> {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        (self.inner as *const I).cmp(&(other.inner as *const I))
    }
}

impl<'a, O, I> PartialEq<Self> for BoxOwned<'a, O, &'a I, ByAddress> {
    fn eq(&self, other: &Self) -> bool {
        ptr::eq(self.inner as *const I, other.inner as *const I)
    }
}

impl<'a, O, I> Eq for BoxOwned<'a, O, &'a I, ByAddress> {}

impl<'a, O, I> PartialOrd<Self> for BoxOwned<'a, O, &'a I, ByAddress> {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        (self.inner as *const I).partial_cmp(&(other.inner as *const I))
    }
}

impl<'a, O, I> Ord for BoxOwned<'a, O, &'a I, ByAddress> {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        (self.inner as *const I).cmp(&(other.inner as *const I))
    }
}

impl<'a, O, I, E> AsRef<I> for BoxOwned<'a, O, I, E>
where
    E: EqKind,
{
    fn as_ref(&self) -> &I {
        self.deref()
    }
}

impl<'a, O, I, E> AsMut<I> for BoxOwned<'a, O, I, E>
where
    E: EqKind,
{
    fn as_mut(&mut self) -> &mut I {
        self.deref_mut()
    }
}

impl<'a, O, I, E> Deref for BoxOwned<'a, O, I, E>
where
    E: EqKind,
{
    type Target = I;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'a, O, I, E> DerefMut for BoxOwned<'a, O, I, E>
where
    E: EqKind,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<'a, O, E> From<Box<O>> for BoxOwned<'a, O, &'a mut O, E>
where
    E: EqKind,
{
    fn from(mut owner: Box<O>) -> Self {
        unsafe {
            // re-borrow to obtain 'a lifetime
            let inner = &mut *(owner.as_mut() as *mut O);

            Self {
                inner,
                owner,
                _phantom: PhantomData,
            }
        }
    }
}
