use crate::marker::*;
use std::{
    cmp, fmt,
    fmt::{Debug, Display},
    marker::PhantomData,
    ops::Deref,
    ptr,
    sync::Arc,
};

pub type ArcRefC<'a, O, I = O> = ArcRef<'a, O, I, ByContent>;
pub type ArcRefA<'a, O, I = O> = ArcRef<'a, O, I, ByAddress>;

pub struct ArcRef<'a, O, I, E>
where
    O: 'a,
    E: EqKind,
{
    // inner goes before owner so that inner drops before owner
    pub(crate) _phantom: PhantomData<E>,
    pub(crate) inner: &'a I,
    pub(crate) owner: Arc<O>,
}

impl<'a, O, E> ArcRef<'a, O, O, E>
where
    O: 'a,
    E: EqKind,
{
    pub fn new(owner: O) -> Self {
        Arc::new(owner).into()
    }

    pub fn from_arc(owner: Arc<O>) -> Self {
        owner.into()
    }
}

impl<'a, O, I, E> ArcRef<'a, O, I, E>
where
    O: 'a,
    E: EqKind,
{
    pub fn into_arc(from: ArcRef<'a, O, I, E>) -> Arc<O> {
        let Self { owner, .. } = from;
        owner
    }

    pub fn into_owner_ref(this: ArcRef<'a, O, I, E>) -> ArcRef<'a, O, O, E> {
        let Self { owner, .. } = this;

        unsafe {
            // re-borrow to obtain 'a lifetime
            let inner = &*(owner.as_ref() as *const O);

            ArcRef {
                inner,
                owner,
                _phantom: PhantomData,
            }
        }
    }

    pub fn owner(this: &'a ArcRef<'a, O, I, E>) -> &'a O {
        &this.owner
    }

    pub fn try_unwrap_owner(from: ArcRef<'a, O, I, E>) -> Option<O> {
        let Self { owner, .. } = from;
        Arc::try_unwrap(owner).ok()
    }

    pub fn unwrap_owner(from: ArcRef<'a, O, I, E>) -> O {
        Self::try_unwrap_owner(from).unwrap()
    }

    pub fn strong_count(this: &ArcRef<'a, O, I, E>) -> usize {
        Arc::strong_count(&this.owner)
    }

    pub fn weak_count(this: &ArcRef<'a, O, I, E>) -> usize {
        Arc::weak_count(&this.owner)
    }

    pub fn map<T, F>(self, f: F) -> ArcRef<'a, O, T, E>
    where
        F: FnOnce(&'a I) -> &'a T,
    {
        let Self { owner, inner, .. } = self;

        ArcRef {
            owner,
            inner: f(inner),
            _phantom: PhantomData,
        }
    }

    pub fn try_map<Ok, Err, F>(self, f: F) -> Result<ArcRef<'a, O, Ok, E>, Err>
    where
        F: FnOnce(&'a I) -> Result<&'a Ok, Err>,
    {
        let Self { owner, inner, .. } = self;

        Ok(ArcRef {
            owner,
            inner: f(inner)?,
            _phantom: PhantomData,
        })
    }

    pub fn filter_map<T, F>(self, f: F) -> Option<ArcRef<'a, O, T, E>>
    where
        F: FnOnce(&'a I) -> Option<&'a T>,
    {
        let Self { owner, inner, .. } = self;

        Some(ArcRef {
            owner,
            inner: f(inner)?,
            _phantom: PhantomData,
        })
    }

    pub fn flatten<T>(self) -> impl IntoIterator<Item = ArcRef<'a, O, T, E>>
    where
        &'a I: IntoIterator<Item = &'a T>,
        T: 'a,
    {
        let Self { owner, inner, .. } = self;
        inner.into_iter().map(move |item| {
            let owner = owner.clone();

            ArcRef {
                owner,
                inner: item,
                _phantom: PhantomData,
            }
        })
    }

    pub fn flat_map<T, C, F>(self, f: F) -> impl IntoIterator<Item = ArcRef<'a, O, T, E>>
    where
        F: FnOnce(&'a I) -> C,
        C: IntoIterator<Item = &'a T>,
        T: 'a,
    {
        let Self { owner, inner, .. } = self;
        let iter = f(inner);

        iter.into_iter().map(move |item| {
            let owner = owner.clone();

            ArcRef {
                owner,
                inner: item,
                _phantom: PhantomData,
            }
        })
    }
}

impl<'a, O, I, E> Clone for ArcRef<'a, O, I, E>
where
    O: 'a,
    I: 'a,
    E: EqKind,
{
    fn clone(&self) -> Self {
        let Self { owner, inner, .. } = self;

        Self {
            owner: owner.clone(),
            inner,
            _phantom: PhantomData,
        }
    }
}

impl<'a, O, I, E> Debug for ArcRef<'a, O, I, E>
where
    O: 'a,
    I: Debug,
    E: EqKind,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        self.inner.fmt(f)
    }
}

impl<'a, O, I, E> Display for ArcRef<'a, O, I, E>
where
    O: 'a,
    I: Display,
    E: EqKind,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        self.inner.fmt(f)
    }
}

impl<'a, O, I> PartialEq<Self> for ArcRef<'a, O, I, ByContent>
where
    O: 'a,
    I: PartialEq<I>,
{
    fn eq(&self, other: &Self) -> bool {
        self.inner.eq(other.inner)
    }
}

impl<'a, O, I> Eq for ArcRef<'a, O, I, ByContent>
where
    O: 'a,
    I: Eq,
{
}

impl<'a, O, I> PartialOrd<Self> for ArcRef<'a, O, I, ByContent>
where
    O: 'a,
    I: PartialOrd<I>,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        self.inner.partial_cmp(other.inner)
    }
}

impl<'a, O, I> Ord for ArcRef<'a, O, I, ByContent>
where
    O: 'a,
    I: Ord,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.inner.cmp(other.inner)
    }
}

impl<'a, O, I> PartialEq<Self> for ArcRef<'a, O, I, ByAddress>
where
    O: 'a,
{
    fn eq(&self, other: &Self) -> bool {
        ptr::eq(self.inner as *const I, other.inner as *const I)
    }
}

impl<'a, O, I> Eq for ArcRef<'a, O, I, ByAddress> where O: 'a {}

impl<'a, O, I> PartialOrd<Self> for ArcRef<'a, O, I, ByAddress>
where
    O: 'a,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        (self.inner as *const I).partial_cmp(&(other.inner as *const I))
    }
}

impl<'a, O, I> Ord for ArcRef<'a, O, I, ByAddress>
where
    O: 'a,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        (self.inner as *const I).cmp(&(other.inner as *const I))
    }
}

impl<'a, O, I, E> AsRef<I> for ArcRef<'a, O, I, E>
where
    O: 'a,
    E: EqKind,
{
    fn as_ref(&self) -> &I {
        self.deref()
    }
}

impl<'a, O, I, E> Deref for ArcRef<'a, O, I, E>
where
    O: 'a,
    E: EqKind,
{
    type Target = I;

    fn deref(&self) -> &Self::Target {
        self.inner
    }
}

impl<'a, O, E> From<Arc<O>> for ArcRef<'a, O, O, E>
where
    O: 'a,
    E: EqKind,
{
    fn from(owner: Arc<O>) -> Self {
        unsafe {
            // re-borrow to obtain 'a lifetime
            let inner = &*(owner.as_ref() as *const O);

            Self {
                inner,
                owner,
                _phantom: PhantomData,
            }
        }
    }
}
