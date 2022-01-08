use crate::{arc_owned::ArcOwned, arc_ref::ArcRef, box_owned::BoxOwned, marker::*};
use std::{
    cmp, fmt,
    fmt::{Debug, Display},
    marker::PhantomData,
    ops::{Deref, DerefMut},
    ptr,
};

pub type BoxRefC<'a, O, I = O> = BoxRef<'a, O, I, ByContent>;
pub type BoxRefA<'a, O, I = O> = BoxRef<'a, O, I, ByAddress>;

pub struct BoxRef<'a, O, I, E>
where
    E: EqKind,
{
    // inner goes before owner so that inner drops before owner
    pub(crate) _phantom: PhantomData<E>,
    pub(crate) inner: &'a mut I,
    pub(crate) owner: Box<O>,
}

impl<'a, O, E> BoxRef<'a, O, O, E>
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

impl<'a, O, I, E> BoxRef<'a, O, I, E>
where
    E: EqKind,
{
    pub fn into_box(from: BoxRef<'a, O, I, E>) -> Box<O> {
        let Self { owner, .. } = from;
        owner
    }

    pub fn into_owner(from: BoxRef<'a, O, I, E>) -> O {
        let Self { owner, .. } = from;
        *owner
    }

    pub fn into_box_owned(from: BoxRef<'a, O, I, E>) -> BoxOwned<'a, O, &mut I, E> {
        let Self { owner, inner, .. } = from;
        BoxOwned {
            owner,
            inner,
            _phantom: PhantomData,
        }
    }

    pub fn into_arc_owned(from: BoxRef<'a, O, I, E>) -> ArcOwned<'a, O, &'a mut I, E> {
        let Self { owner, inner, .. } = from;
        ArcOwned {
            owner: owner.into(),
            inner,
            _phantom: PhantomData,
        }
    }

    pub fn into_arc_ref(from: BoxRef<'a, O, I, E>) -> ArcRef<'a, O, I, E> {
        let Self { owner, inner, .. } = from;
        ArcRef {
            owner: owner.into(),
            inner,
            _phantom: PhantomData,
        }
    }

    pub fn into_owner_ref(this: BoxRef<'a, O, I, E>) -> BoxRef<'a, O, O, E> {
        let Self { mut owner, .. } = this;

        unsafe {
            // re-borrow to obtain 'a lifetime
            let inner = &mut *(owner.as_mut() as *mut O);

            BoxRef {
                inner,
                owner,
                _phantom: PhantomData,
            }
        }
    }

    pub fn owner(this: &'a BoxRef<'a, O, I, E>) -> &'a O {
        &this.owner
    }

    pub fn map<T, F>(self, f: F) -> BoxRef<'a, O, T, E>
    where
        F: FnOnce(&'a mut I) -> &'a mut T,
    {
        let Self { owner, inner, .. } = self;

        BoxRef {
            owner,
            inner: f(inner),
            _phantom: PhantomData,
        }
    }

    pub fn try_map<Ok, Err, F>(self, f: F) -> Result<BoxRef<'a, O, Ok, E>, Err>
    where
        F: FnOnce(&'a mut I) -> Result<&'a mut Ok, Err>,
    {
        let Self { owner, inner, .. } = self;

        Ok(BoxRef {
            owner,
            inner: f(inner)?,
            _phantom: PhantomData,
        })
    }

    pub fn filter_map<T, F>(self, f: F) -> Option<BoxRef<'a, O, T, E>>
    where
        F: FnOnce(&'a mut I) -> Option<&'a mut T>,
    {
        let Self { owner, inner, .. } = self;

        Some(BoxRef {
            owner,
            inner: f(inner)?,
            _phantom: PhantomData,
        })
    }
}

impl<'a, O, I, E> Debug for BoxRef<'a, O, I, E>
where
    I: Debug,
    E: EqKind,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        self.inner.fmt(f)
    }
}

impl<'a, O, I, E> Display for BoxRef<'a, O, I, E>
where
    I: Display,
    E: EqKind,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        self.inner.fmt(f)
    }
}

impl<'a, O, I> PartialEq<Self> for BoxRef<'a, O, I, ByContent>
where
    I: PartialEq<I>,
{
    fn eq(&self, other: &Self) -> bool {
        self.inner.eq(&other.inner)
    }
}

impl<'a, O, I> Eq for BoxRef<'a, O, I, ByContent> where I: Eq {}

impl<'a, O, I> PartialOrd<Self> for BoxRef<'a, O, I, ByContent>
where
    I: PartialOrd<I>,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        self.inner.partial_cmp(&other.inner)
    }
}

impl<'a, O, I> Ord for BoxRef<'a, O, I, ByContent>
where
    I: Ord,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.inner.cmp(&other.inner)
    }
}

impl<'a, O, I> PartialEq<Self> for BoxRef<'a, O, I, ByAddress> {
    fn eq(&self, other: &Self) -> bool {
        ptr::eq(self.inner as *const I, other.inner as *const I)
    }
}

impl<'a, O, I> Eq for BoxRef<'a, O, I, ByAddress> {}

impl<'a, O, I> PartialOrd<Self> for BoxRef<'a, O, I, ByAddress> {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        (self.inner as *const I).partial_cmp(&(other.inner as *const I))
    }
}

impl<'a, O, I> Ord for BoxRef<'a, O, I, ByAddress> {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        (self.inner as *const I).cmp(&(other.inner as *const I))
    }
}

impl<'a, O, I, E> AsRef<I> for BoxRef<'a, O, I, E>
where
    E: EqKind,
{
    fn as_ref(&self) -> &I {
        self.deref()
    }
}

impl<'a, O, I, E> AsMut<I> for BoxRef<'a, O, I, E>
where
    E: EqKind,
{
    fn as_mut(&mut self) -> &mut I {
        self.deref_mut()
    }
}

impl<'a, O, I, E> Deref for BoxRef<'a, O, I, E>
where
    E: EqKind,
{
    type Target = I;

    fn deref(&self) -> &Self::Target {
        self.inner
    }
}

impl<'a, O, I, E> DerefMut for BoxRef<'a, O, I, E>
where
    E: EqKind,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner
    }
}

impl<'a, O, E> From<Box<O>> for BoxRef<'a, O, O, E>
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