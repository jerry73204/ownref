use crate::{arc_owned::ArcOwned, marker::*};
use std::{
    any::Any,
    borrow::Borrow,
    cmp, fmt,
    fmt::{Debug, Display},
    hash::{Hash, Hasher},
    marker::PhantomData,
    ops::Deref,
    ptr,
    sync::Arc,
};

/// Content ordered reference to data within an owner in [Arc].
pub type ArcRefC<'a, O, I = O> = ArcRef<'a, O, I, ByContent>;

/// Pointer address ordered reference to data within an owner in [Arc].
pub type ArcRefA<'a, O, I = O> = ArcRef<'a, O, I, ByAddress>;

/// Content ordered reference to data within an [Any] owner in [Arc].
pub type ArcRefAnyC<'a, I> = ArcRef<'a, dyn Any + Send + Sync + 'static, I, ByContent>;

/// Pointer address ordered reference to data within an [Any] owner in [Arc].
pub type ArcRefAnyA<'a, I> = ArcRef<'a, dyn Any + Send + Sync + 'static, I, ByAddress>;

/// Reference to data within an owner in [Arc].
pub struct ArcRef<'a, O, I, E>
where
    O: ?Sized,
    I: ?Sized,
    E: EqKind,
{
    // inner goes before owner so that inner drops before owner
    pub(crate) _phantom: PhantomData<E>,
    pub(crate) inner: &'a I,
    pub(crate) owner: Arc<O>,
}

impl<'a, O, E> ArcRef<'a, O, O, E>
where
    O: ?Sized,
    E: EqKind,
{
    /// Build from owner data in [Arc].
    pub fn from_arc(owner: Arc<O>) -> Self {
        owner.into()
    }
}

impl<'a, O, I, E> ArcRef<'a, O, I, E>
where
    O: ?Sized,
    I: ?Sized,
    E: EqKind,
{
    /// Discard the inner reference and return the owner in [Arc].
    pub fn into_arc(from: ArcRef<'a, O, I, E>) -> Arc<O> {
        let Self { owner, .. } = from;
        owner
    }

    /// Convert to [ArcOwned].
    pub fn into_arc_owned(this: ArcRef<'a, O, I, E>) -> ArcOwned<'a, O, &'a I, E> {
        let Self { owner, inner, .. } = this;
        ArcOwned {
            inner,
            owner,
            _phantom: PhantomData,
        }
    }

    /// Reset the inner reference to the owner.
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

    /// Get the reference to the owner.
    pub fn owner(this: &'a ArcRef<'a, O, I, E>) -> &'a O {
        &this.owner
    }

    /// Get the strong count on the owner.
    pub fn strong_count(this: &ArcRef<'a, O, I, E>) -> usize {
        Arc::strong_count(&this.owner)
    }

    /// Get the weak count on the owner.
    pub fn weak_count(this: &ArcRef<'a, O, I, E>) -> usize {
        Arc::weak_count(&this.owner)
    }

    /// Apply function `f` to the inner reference.
    pub fn map<T, F>(self, f: F) -> ArcRef<'a, O, T, E>
    where
        F: FnOnce(&'a I) -> &'a T,
        T: ?Sized,
    {
        let Self { owner, inner, .. } = self;

        ArcRef {
            owner,
            inner: f(inner),
            _phantom: PhantomData,
        }
    }

    /// Apply fallible function `f` to the inner reference.
    pub fn try_map<Ok, Err, F>(self, f: F) -> Result<ArcRef<'a, O, Ok, E>, Err>
    where
        F: FnOnce(&'a I) -> Result<&'a Ok, Err>,
        Ok: ?Sized,
    {
        let Self { owner, inner, .. } = self;

        Ok(ArcRef {
            owner,
            inner: f(inner)?,
            _phantom: PhantomData,
        })
    }

    /// Apply function `f` that returns an optional reference to the inner reference.
    pub fn filter_map<T, F>(self, f: F) -> Option<ArcRef<'a, O, T, E>>
    where
        F: FnOnce(&'a I) -> Option<&'a T>,
        T: ?Sized,
    {
        let Self { owner, inner, .. } = self;

        Some(ArcRef {
            owner,
            inner: f(inner)?,
            _phantom: PhantomData,
        })
    }

    /// Flatten the wrapped iterable inner reference into an iterator of wrapped items.
    pub fn flatten<T>(self) -> impl Iterator<Item = ArcRef<'a, O, T, E>>
    where
        &'a I: IntoIterator<Item = &'a T>,
        T: 'a + ?Sized,
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

    /// Apply fucntion `f` to get an iterable type, and flatten it to an iterator of references.
    pub fn flat_map<T, C, F>(self, f: F) -> impl Iterator<Item = ArcRef<'a, O, T, E>>
    where
        F: FnOnce(&'a I) -> C,
        C: IntoIterator<Item = &'a T>,
        T: 'a + ?Sized,
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

impl<'a, O, I, E> ArcRef<'a, O, I, E>
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

    /// Convert the owner type to [Any] trait object.
    pub fn into_any_owner(
        from: ArcRef<'a, O, I, E>,
    ) -> ArcRef<'a, dyn Any + Send + Sync + 'static, I, E>
    where
        O: Send + Sync + 'static,
    {
        let Self { owner, inner, .. } = from;

        ArcRef {
            inner,
            owner,
            _phantom: PhantomData,
        }
    }

    /// Unwrap the owner if strong count is one.
    pub fn try_unwrap_owner(from: ArcRef<'a, O, I, E>) -> Result<O, Self> {
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
    pub fn unwrap_owner(from: ArcRef<'a, O, I, E>) -> O {
        Self::try_unwrap_owner(from)
            .unwrap_or_else(|_| panic!("unable to unwrap because strong count is greater than 1"))
    }
}

impl<'a, I, E> ArcRef<'a, dyn Any + Send + Sync + 'static, I, E>
where
    I: ?Sized,
    E: EqKind,
{
    /// Downcast the [Any]-trait object owner to concrete type.
    pub fn downcast_owner<O>(this: Self) -> Result<ArcRef<'a, O, I, E>, Self>
    where
        O: Send + Sync + 'static,
    {
        let Self { owner, inner, .. } = this;

        match owner.downcast() {
            Ok(owner) => Ok(ArcRef {
                owner,
                inner,
                _phantom: PhantomData,
            }),
            Err(owner) => Err(ArcRef {
                owner,
                inner,
                _phantom: PhantomData,
            }),
        }
    }
}

impl<'a, O, I, E> Clone for ArcRef<'a, O, I, E>
where
    O: ?Sized,
    I: ?Sized,
    E: EqKind,
{
    /// Copy the inner reference and increase reference count to owner.
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
    O: ?Sized,
    I: ?Sized,
    I: Debug,
    E: EqKind,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        self.inner.fmt(f)
    }
}

impl<'a, O, I, E> Display for ArcRef<'a, O, I, E>
where
    O: ?Sized,
    I: ?Sized,
    I: Display,
    E: EqKind,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        self.inner.fmt(f)
    }
}

impl<'a, O, I> PartialEq<Self> for ArcRef<'a, O, I, ByContent>
where
    O: ?Sized,
    I: ?Sized,
    I: PartialEq<I>,
{
    fn eq(&self, other: &Self) -> bool {
        self.inner.eq(other.inner)
    }
}

impl<'a, O, I> Eq for ArcRef<'a, O, I, ByContent>
where
    I: Eq,
    O: ?Sized,
    I: ?Sized,
{
}

impl<'a, O, I> PartialOrd<Self> for ArcRef<'a, O, I, ByContent>
where
    O: ?Sized,
    I: ?Sized,
    I: PartialOrd<I>,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        self.inner.partial_cmp(other.inner)
    }
}

impl<'a, O, I> Ord for ArcRef<'a, O, I, ByContent>
where
    O: ?Sized,
    I: ?Sized,
    I: Ord,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.inner.cmp(other.inner)
    }
}

impl<'a, O, I> Hash for ArcRef<'a, O, I, ByContent>
where
    O: ?Sized,
    I: ?Sized + Hash,
{
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.inner.hash(state);
    }
}

impl<'a, O, I> PartialEq<Self> for ArcRef<'a, O, I, ByAddress>
where
    O: ?Sized,
    I: ?Sized,
{
    fn eq(&self, other: &Self) -> bool {
        ptr::eq(self.inner as *const I, other.inner as *const I)
    }
}

impl<'a, O, I> Eq for ArcRef<'a, O, I, ByAddress>
where
    O: ?Sized,
    I: ?Sized,
{
}

impl<'a, O, I> PartialOrd<Self> for ArcRef<'a, O, I, ByAddress>
where
    O: ?Sized,
    I: ?Sized,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        (self.inner as *const I).partial_cmp(&(other.inner as *const I))
    }
}

impl<'a, O, I> Ord for ArcRef<'a, O, I, ByAddress>
where
    O: ?Sized,
    I: ?Sized,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        (self.inner as *const I).cmp(&(other.inner as *const I))
    }
}

impl<'a, O, I> Hash for ArcRef<'a, O, I, ByAddress>
where
    O: ?Sized,
    I: ?Sized,
{
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        ptr::hash(self.inner as *const I, state);
    }
}

impl<'a, O, I, E> AsRef<I> for ArcRef<'a, O, I, E>
where
    O: ?Sized,
    I: ?Sized,
    E: EqKind,
{
    fn as_ref(&self) -> &I {
        self.deref()
    }
}

impl<'a, O, I, E> Borrow<I> for ArcRef<'a, O, I, E>
where
    O: ?Sized,
    I: ?Sized,
    E: EqKind,
{
    fn borrow(&self) -> &I {
        self.deref()
    }
}

impl<'a, O, I, E> Deref for ArcRef<'a, O, I, E>
where
    O: ?Sized,
    I: ?Sized,
    E: EqKind,
{
    type Target = I;

    fn deref(&self) -> &Self::Target {
        self.inner
    }
}

impl<'a, O, E> From<Arc<O>> for ArcRef<'a, O, O, E>
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
