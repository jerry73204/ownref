use crate::{arc_ref::ArcRef, marker::*};
use std::{
    any::Any,
    cmp, fmt,
    fmt::{Debug, Display},
    marker::PhantomData,
    ops::Deref,
    ptr,
    sync::Arc,
};

pub type ArcOwnedC<'a, O, I = &'a O> = ArcOwned<'a, O, I, ByContent>;
pub type ArcOwnedA<'a, O, I = &'a O> = ArcOwned<'a, O, I, ByAddress>;
pub type ArcOwnedAnyC<'a, I> = ArcOwned<'a, dyn Any + Send + Sync + 'static, I, ByContent>;
pub type ArcOwnedAnyA<'a, I> = ArcOwned<'a, dyn Any + Send + Sync + 'static, I, ByAddress>;

pub struct ArcOwned<'a, O, I, E>
where
    O: ?Sized,
    E: EqKind,
{
    // inner goes before owner so that inner drops before owner
    pub(crate) _phantom: PhantomData<(&'a I, E)>,
    pub(crate) inner: I,
    pub(crate) owner: Arc<O>,
}

impl<'a, O, E> ArcOwned<'a, O, &'a O, E>
where
    O: ?Sized,
    E: EqKind,
{
    pub fn from_arc(owner: Arc<O>) -> Self {
        owner.into()
    }
}

impl<'a, O, I, E> ArcOwned<'a, O, I, E>
where
    O: ?Sized,
    E: EqKind,
{
    pub fn into_arc(from: ArcOwned<'a, O, I, E>) -> Arc<O> {
        let Self { owner, inner, .. } = from;
        drop(inner);
        owner
    }

    pub fn into_owner_ref(this: ArcOwned<'a, O, I, E>) -> ArcOwned<'a, O, &O, E> {
        let Self { owner, inner, .. } = this;
        drop(inner);

        unsafe {
            // re-borrow to obtain 'a lifetime
            let inner = &*(owner.as_ref() as *const O);

            ArcOwned {
                inner,
                owner,
                _phantom: PhantomData,
            }
        }
    }

    pub fn owner(this: &'a ArcOwned<'a, O, I, E>) -> &'a O {
        &this.owner
    }

    pub fn strong_count(this: &ArcOwned<'a, O, I, E>) -> usize {
        Arc::strong_count(&this.owner)
    }

    pub fn weak_count(this: &ArcOwned<'a, O, I, E>) -> usize {
        Arc::weak_count(&this.owner)
    }

    pub fn map<T, F>(self, f: F) -> ArcOwned<'a, O, T, E>
    where
        F: FnOnce(I) -> T,
    {
        let Self { owner, inner, .. } = self;

        ArcOwned {
            owner,
            inner: f(inner),
            _phantom: PhantomData,
        }
    }

    pub fn try_map<Ok, Err, F>(self, f: F) -> Result<ArcOwned<'a, O, Ok, E>, Err>
    where
        F: FnOnce(I) -> Result<Ok, Err>,
    {
        let Self { owner, inner, .. } = self;

        Ok(ArcOwned {
            owner,
            inner: f(inner)?,
            _phantom: PhantomData,
        })
    }

    pub fn filter_map<T, F>(self, f: F) -> Option<ArcOwned<'a, O, T, E>>
    where
        F: FnOnce(I) -> Option<T>,
    {
        let Self { owner, inner, .. } = self;

        Some(ArcOwned {
            owner,
            inner: f(inner)?,
            _phantom: PhantomData,
        })
    }

    pub fn flatten(self) -> impl IntoIterator<Item = ArcOwned<'a, O, I::Item, E>>
    where
        I: IntoIterator,
    {
        let Self { owner, inner, .. } = self;
        inner.into_iter().map(move |item| {
            let owner = owner.clone();

            ArcOwned {
                owner,
                inner: item,
                _phantom: PhantomData,
            }
        })
    }

    pub fn flat_map<T, F>(self, f: F) -> impl IntoIterator<Item = ArcOwned<'a, O, T::Item, E>>
    where
        F: FnOnce(I) -> T,
        T: 'a + IntoIterator,
    {
        self.map(f).flatten()
    }
}

impl<'a, O, I, E> ArcOwned<'a, O, I, E>
where
    E: EqKind,
{
    pub fn new(owner: O) -> Self
    where
        Self: From<Arc<O>>,
    {
        Arc::new(owner).into()
    }

    pub fn into_any_owner(
        from: ArcOwned<'a, O, I, E>,
    ) -> ArcOwned<'a, dyn Any + Send + Sync + 'static, I, E>
    where
        O: Send + Sync + 'static,
    {
        let Self { owner, inner, .. } = from;

        ArcOwned {
            inner,
            owner,
            _phantom: PhantomData,
        }
    }

    pub fn try_unwrap_owner(from: ArcOwned<'a, O, I, E>) -> Option<O> {
        let Self { owner, inner, .. } = from;
        drop(inner);
        Arc::try_unwrap(owner).ok()
    }

    pub fn unwrap_owner(from: ArcOwned<'a, O, I, E>) -> O {
        Self::try_unwrap_owner(from).unwrap()
    }
}

impl<'a, O, I, E> ArcOwned<'a, O, &'a I, E>
where
    O: ?Sized,
    E: EqKind,
{
    pub fn into_arc_ref(this: ArcOwned<'a, O, &'a I, E>) -> ArcRef<'a, O, I, E> {
        let Self { owner, inner, .. } = this;

        ArcRef {
            owner,
            inner,
            _phantom: PhantomData,
        }
    }
}

impl<'a, O, I, E> ArcOwned<'a, O, Option<I>, E>
where
    O: ?Sized,
    E: EqKind,
{
    pub fn transpose(self) -> Option<ArcOwned<'a, O, I, E>> {
        let Self { owner, inner, .. } = self;
        Some(ArcOwned {
            owner,
            inner: inner?,
            _phantom: PhantomData,
        })
    }
}

impl<'a, O, Ok, Err, E> ArcOwned<'a, O, Result<Ok, Err>, E>
where
    O: ?Sized,
    E: EqKind,
{
    pub fn transpose(self) -> Result<ArcOwned<'a, O, Ok, E>, Err> {
        let Self { owner, inner, .. } = self;
        Ok(ArcOwned {
            owner,
            inner: inner?,
            _phantom: PhantomData,
        })
    }
}

impl<'a, I, E> ArcOwned<'a, dyn Any + Send + Sync + 'static, I, E>
where
    E: EqKind,
{
    pub fn downcast_owner<O>(
        this: ArcOwned<'a, dyn Any + Send + Sync + 'static, I, E>,
    ) -> Result<ArcOwned<'a, O, I, E>, ArcOwned<'a, dyn Any + Send + Sync + 'static, I, E>>
    where
        O: Send + Sync + 'static,
    {
        let Self { owner, inner, .. } = this;

        match owner.downcast() {
            Ok(owner) => Ok(ArcOwned {
                owner,
                inner,
                _phantom: PhantomData,
            }),
            Err(owner) => Err(ArcOwned {
                owner,
                inner,
                _phantom: PhantomData,
            }),
        }
    }
}

impl<'a, O, I, E> Clone for ArcOwned<'a, O, I, E>
where
    O: ?Sized,
    I: Clone,
    E: EqKind,
{
    fn clone(&self) -> Self {
        let Self { owner, inner, .. } = self;

        Self {
            owner: owner.clone(),
            inner: inner.clone(),
            _phantom: PhantomData,
        }
    }
}

impl<'a, O, I, E> Debug for ArcOwned<'a, O, I, E>
where
    O: ?Sized,
    I: Debug,
    E: EqKind,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        self.inner.fmt(f)
    }
}

impl<'a, O, I, E> Display for ArcOwned<'a, O, I, E>
where
    O: ?Sized,
    I: Display,
    E: EqKind,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        self.inner.fmt(f)
    }
}

impl<'a, O, I> PartialEq<Self> for ArcOwned<'a, O, I, ByContent>
where
    O: ?Sized,
    I: PartialEq<I>,
{
    fn eq(&self, other: &Self) -> bool {
        self.inner.eq(&other.inner)
    }
}

impl<'a, O, I> Eq for ArcOwned<'a, O, I, ByContent>
where
    I: Eq,
    O: ?Sized,
{
}

impl<'a, O, I> PartialOrd<Self> for ArcOwned<'a, O, I, ByContent>
where
    O: ?Sized,
    I: PartialOrd<I>,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        self.inner.partial_cmp(&other.inner)
    }
}

impl<'a, O, I> Ord for ArcOwned<'a, O, I, ByContent>
where
    O: ?Sized,
    I: Ord,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.inner.cmp(&other.inner)
    }
}

impl<'a, O, I> PartialEq<Self> for ArcOwned<'a, O, &'a I, ByAddress>
where
    O: ?Sized,
{
    fn eq(&self, other: &Self) -> bool {
        ptr::eq(self.inner as *const I, other.inner as *const I)
    }
}

impl<'a, O, I> Eq for ArcOwned<'a, O, &'a I, ByAddress> where O: ?Sized {}

impl<'a, O, I> PartialOrd<Self> for ArcOwned<'a, O, &'a I, ByAddress>
where
    O: ?Sized,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        (self.inner as *const I).partial_cmp(&(other.inner as *const I))
    }
}

impl<'a, O, I> Ord for ArcOwned<'a, O, &'a I, ByAddress>
where
    O: ?Sized,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        (self.inner as *const I).cmp(&(other.inner as *const I))
    }
}

impl<'a, O, I, E> AsRef<I> for ArcOwned<'a, O, I, E>
where
    O: ?Sized,
    E: EqKind,
{
    fn as_ref(&self) -> &I {
        self.deref()
    }
}

impl<'a, O, I, E> Deref for ArcOwned<'a, O, I, E>
where
    O: ?Sized,
    E: EqKind,
{
    type Target = I;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'a, O, E> From<Arc<O>> for ArcOwned<'a, O, &'a O, E>
where
    O: ?Sized,
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
