use crate::{arc_ref::ArcRef, marker::*};
use std::{
    any::Any,
    borrow::Borrow,
    cmp, fmt,
    fmt::{Debug, Display},
    future::Future,
    hash::{Hash, Hasher},
    marker::PhantomData,
    ops::Deref,
    ptr,
    sync::Arc,
};

/// Content ordered owned data bundled with an owner in [Arc].
pub type ArcOwnedC<'a, O, I = &'a O> = ArcOwned<'a, O, I, ByContent>;

/// Pointer address ordered owned data bundled with an owner in [Arc].
pub type ArcOwnedA<'a, O, I = &'a O> = ArcOwned<'a, O, I, ByAddress>;

/// Content ordered owned data bundled with an [Any] owner in [Arc].
pub type ArcOwnedAnyC<'a, I> = ArcOwned<'a, dyn Any + Send + Sync + 'static, I, ByContent>;

/// Pointer address ordered owned data bundled with an [Any] owner in [Arc].
pub type ArcOwnedAnyA<'a, I> = ArcOwned<'a, dyn Any + Send + Sync + 'static, I, ByAddress>;

/// Owned data bundled with an owner in [Arc].
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
    /// Discard data and return owner in [Arc].
    pub fn into_arc(from: ArcOwned<'a, O, I, E>) -> Arc<O> {
        let Self { owner, inner, .. } = from;
        drop(inner);
        owner
    }

    /// Reset data to reference to owner.
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

    /// Get reference to owner.
    pub fn owner(this: &'a ArcOwned<'a, O, I, E>) -> &'a O {
        &this.owner
    }

    /// Get strong count on owner.
    pub fn strong_count(this: &ArcOwned<'a, O, I, E>) -> usize {
        Arc::strong_count(&this.owner)
    }

    /// Get weak count on owner.
    pub fn weak_count(this: &ArcOwned<'a, O, I, E>) -> usize {
        Arc::weak_count(&this.owner)
    }

    /// Applies function `f` to data.
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

    /// Applies fallible function `f` to data.
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

    /// Applies fallible function `f` to data.
    pub async fn try_then<Ok, Err, F, Fut>(self, f: F) -> Result<ArcOwned<'a, O, Ok, E>, Err>
    where
        Ok: 'a,
        F: FnOnce(I) -> Fut,
        Fut: Future<Output = Result<Ok, Err>>,
    {
        let Self { owner, inner, .. } = self;

        Ok(ArcOwned {
            owner,
            inner: f(inner).await?,
            _phantom: PhantomData,
        })
    }

    /// Applies function `f` that returns optional value to data.
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

    /// Applies function `f` that returns optional value to data.
    pub async fn filter_then<T, F, Fut>(self, f: F) -> Option<ArcOwned<'a, O, T, E>>
    where
        T: 'a,
        F: FnOnce(I) -> Fut,
        Fut: Future<Output = Option<T>>,
    {
        let Self { owner, inner, .. } = self;

        Some(ArcOwned {
            owner,
            inner: f(inner).await?,
            _phantom: PhantomData,
        })
    }

    /// Flatten the wrapped iterable data into an iterator of wrapped items.
    pub fn flatten(self) -> impl Iterator<Item = ArcOwned<'a, O, I::Item, E>>
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

    /// Apply fucntion `f` to get an iterable type, and flatten it to an iterator of wrapped items.
    pub fn flat_map<T, F>(self, f: F) -> impl Iterator<Item = ArcOwned<'a, O, T::Item, E>>
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
    /// Build from an owner.
    pub fn new(owner: O) -> Self
    where
        Self: From<Arc<O>>,
    {
        Arc::new(owner).into()
    }

    /// Change the owner type to [Any] trait object.
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

    /// Unwrap the owner if strong count is one.
    pub fn try_unwrap_owner(from: ArcOwned<'a, O, I, E>) -> Result<O, Self> {
        let Self { owner, inner, .. } = from;

        match Arc::try_unwrap(owner) {
            Ok(owner) => Ok(owner),
            Err(owner) => Err(Self {
                owner,
                inner,
                _phantom: PhantomData,
            }),
        }
    }

    /// Unwrap the owner and panic if strong count is one.
    ///
    /// # Panic
    /// The method panics if strong count is not 1.
    pub fn unwrap_owner(from: ArcOwned<'a, O, I, E>) -> O {
        Self::try_unwrap_owner(from)
            .unwrap_or_else(|_| panic!("unable to unwrap because strong count is greater than 1"))
    }
}

impl<'a, O, I, E> ArcOwned<'a, O, &'a I, E>
where
    O: ?Sized,
    I: ?Sized,
    E: EqKind,
{
    /// Convert ot [ArcRef].
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
    /// Transpose an [ArcOwned] of an [Option] to an [Option] of an [ArcOwned].
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
    /// Transpose an [ArcOwned] of a [Result] to a [Result] of an [ArcOwned].
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
    /// Downcast the [Any]-trait object owner to concrete type.
    pub fn downcast_owner<O>(this: Self) -> Result<ArcOwned<'a, O, I, E>, Self>
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
    /// Clone the data and increase reference count to owner.
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

impl<'a, O, I> Hash for ArcOwned<'a, O, I, ByContent>
where
    O: ?Sized,
    I: Hash,
{
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.inner.hash(state);
    }
}

impl<'a, O, I> PartialEq<Self> for ArcOwned<'a, O, &'a I, ByAddress>
where
    O: ?Sized,
    I: ?Sized,
{
    fn eq(&self, other: &Self) -> bool {
        ptr::eq(self.inner as *const I, other.inner as *const I)
    }
}

impl<'a, O, I> Eq for ArcOwned<'a, O, &'a I, ByAddress>
where
    O: ?Sized,
    I: ?Sized,
{
}

impl<'a, O, I> PartialOrd<Self> for ArcOwned<'a, O, &'a I, ByAddress>
where
    O: ?Sized,
    I: ?Sized,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        (self.inner as *const I).partial_cmp(&(other.inner as *const I))
    }
}

impl<'a, O, I> Ord for ArcOwned<'a, O, &'a I, ByAddress>
where
    O: ?Sized,
    I: ?Sized,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        (self.inner as *const I).cmp(&(other.inner as *const I))
    }
}

impl<'a, O, I> Hash for ArcOwned<'a, O, &'a I, ByAddress>
where
    O: ?Sized,
{
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        ptr::hash(self.inner as *const I, state);
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

impl<'a, O, I, E> Borrow<I> for ArcOwned<'a, O, I, E>
where
    O: ?Sized,
    E: EqKind,
{
    fn borrow(&self) -> &I {
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
