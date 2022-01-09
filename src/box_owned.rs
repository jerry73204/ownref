use crate::{arc_owned::ArcOwned, arc_ref::ArcRef, box_ref::BoxRef, marker::*};
use std::{
    any::Any,
    borrow::Borrow,
    cmp, fmt,
    fmt::{Debug, Display},
    hash::{Hash, Hasher},
    marker::PhantomData,
    ops::{Deref, DerefMut},
    ptr,
};

/// Content ordered owned data bundled with an owner in [Box].
pub type BoxOwnedC<'a, O, I = &'a mut O> = BoxOwned<'a, O, I, ByContent>;

/// Pointer address ordered owned data bundled with an owner in [Box].
pub type BoxOwnedA<'a, O, I = &'a mut O> = BoxOwned<'a, O, I, ByAddress>;

/// Content ordered owned data bundled with an [Any] + [Send] owner in [Box].
pub type BoxOwnedAnyC<'a, I> = BoxOwned<'a, dyn Any + Send + 'static, I, ByContent>;

/// Pointer address ordered owned data bundled with an [Any] + [Send] owner in [Box].
pub type BoxOwnedAnyA<'a, I> = BoxOwned<'a, dyn Any + Send + 'static, I, ByAddress>;

/// Content ordered owned data bundled with an [Any] owner in [Box].
pub type BoxOwnedAnyLocalC<'a, I> = BoxOwned<'a, dyn Any + 'static, I, ByContent>;

/// Pointer address ordered owned data bundled with an [Any] owner in [Box].
pub type BoxOwnedAnyLocalA<'a, I> = BoxOwned<'a, dyn Any + 'static, I, ByAddress>;

/// Owned data bundled with an owner in [Box].
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
    O: ?Sized,
    E: EqKind,
{
    /// Build from boxed data.
    pub fn from_box(owner: Box<O>) -> Self {
        owner.into()
    }
}

impl<'a, O, I, E> BoxOwned<'a, O, I, E>
where
    O: ?Sized,
    E: EqKind,
{
    /// Discard owned data and return boxed owner.
    pub fn into_box(from: BoxOwned<'a, O, I, E>) -> Box<O> {
        let Self { owner, inner, .. } = from;
        drop(inner);
        owner
    }

    /// Convert to [ArcOwned] without re-allocation.
    pub fn into_arc_owned(from: BoxOwned<'a, O, I, E>) -> ArcOwned<'a, O, I, E> {
        let Self { owner, inner, .. } = from;
        ArcOwned {
            owner: owner.into(),
            inner,
            _phantom: PhantomData,
        }
    }

    /// Reset data to the reference to owner.
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

    /// Get the reference to owner.
    pub fn owner(this: &'a BoxOwned<'a, O, I, E>) -> &'a O {
        &this.owner
    }

    /// Applies function `f` to data.
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

    /// Applies fallible function `f` to data.
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

    /// Applies function `f` that returns optional value to data.
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

impl<'a, O, I, E> BoxOwned<'a, O, I, E>
where
    E: EqKind,
{
    /// Build from an owner.
    pub fn new(owner: O) -> Self
    where
        Self: From<Box<O>>,
    {
        Box::new(owner).into()
    }

    /// Discard the data and return the owner.
    pub fn into_owner(from: BoxOwned<'a, O, I, E>) -> O {
        let Self { owner, inner, .. } = from;
        drop(inner);
        *owner
    }

    /// Change the owner type to [Any] + [Send] trait object.
    pub fn into_any_owner(
        from: BoxOwned<'a, O, I, E>,
    ) -> BoxOwned<'a, dyn Any + Send + 'static, I, E>
    where
        O: Send + 'static,
    {
        let Self { owner, inner, .. } = from;

        BoxOwned {
            inner,
            owner,
            _phantom: PhantomData,
        }
    }

    /// Change the owner type to [Any] trait object.
    pub fn into_any_owner_local(
        from: BoxOwned<'a, O, I, E>,
    ) -> BoxOwned<'a, dyn Any + 'static, I, E>
    where
        O: 'static,
    {
        let Self { owner, inner, .. } = from;

        BoxOwned {
            inner,
            owner,
            _phantom: PhantomData,
        }
    }
}

impl<'a, O, I, E> BoxOwned<'a, O, &'a mut I, E>
where
    O: ?Sized,
    I: ?Sized,
    E: EqKind,
{
    /// Convert to [BoxRef].
    pub fn into_box_ref(self) -> BoxRef<'a, O, I, E> {
        let Self { owner, inner, .. } = self;

        BoxRef {
            owner,
            inner,
            _phantom: PhantomData,
        }
    }

    /// Convert to [ArcRef].
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
    O: ?Sized,
    I: ?Sized,
    E: EqKind,
{
    /// Convert to [ArcRef].
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
    O: ?Sized,
    E: EqKind,
{
    /// Transpose an [BoxOwned] of a [Option] to a [Option] of an [BoxOwned].
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
    O: ?Sized,
    E: EqKind,
{
    /// Transpose an [BoxOwned] of a [Result] to a [Result] of an [BoxOwned].
    pub fn transpose(self) -> Result<BoxOwned<'a, O, Ok, E>, Err> {
        let Self { owner, inner, .. } = self;
        Ok(BoxOwned {
            owner,
            inner: inner?,
            _phantom: PhantomData,
        })
    }
}

impl<'a, I, E> BoxOwned<'a, dyn Any + Send + 'static, I, E>
where
    E: EqKind,
{
    /// Downcast the [Any] + [Send] trait object owner to concrete type.
    pub fn downcast_owner<O>(this: Self) -> Result<BoxOwned<'a, O, I, E>, Self>
    where
        O: Send + 'static,
    {
        let Self { owner, inner, .. } = this;

        match owner.downcast() {
            Ok(owner) => Ok(BoxOwned {
                owner,
                inner,
                _phantom: PhantomData,
            }),
            Err(owner) => Err(BoxOwned {
                owner,
                inner,
                _phantom: PhantomData,
            }),
        }
    }
}

impl<'a, I, E> BoxOwned<'a, dyn Any + 'static, I, E>
where
    E: EqKind,
{
    /// Downcast the [Any] trait object owner to concrete type.
    pub fn downcast_owner_local<O>(this: Self) -> Result<BoxOwned<'a, O, I, E>, Self>
    where
        O: 'static,
    {
        let Self { owner, inner, .. } = this;

        match owner.downcast() {
            Ok(owner) => Ok(BoxOwned {
                owner,
                inner,
                _phantom: PhantomData,
            }),
            Err(owner) => Err(BoxOwned {
                owner,
                inner,
                _phantom: PhantomData,
            }),
        }
    }
}

impl<'a, O, I, E> Debug for BoxOwned<'a, O, I, E>
where
    O: ?Sized,
    I: Debug,
    E: EqKind,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        self.inner.fmt(f)
    }
}

impl<'a, O, I, E> Display for BoxOwned<'a, O, I, E>
where
    O: ?Sized,
    I: Display,
    E: EqKind,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        self.inner.fmt(f)
    }
}

impl<'a, O, I> PartialEq<Self> for BoxOwned<'a, O, I, ByContent>
where
    O: ?Sized,
    I: PartialEq<I>,
{
    fn eq(&self, other: &Self) -> bool {
        self.inner.eq(&other.inner)
    }
}

impl<'a, O, I> Eq for BoxOwned<'a, O, I, ByContent>
where
    I: Eq,
    O: ?Sized,
{
}

impl<'a, O, I> PartialOrd<Self> for BoxOwned<'a, O, I, ByContent>
where
    O: ?Sized,
    I: PartialOrd<I>,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        self.inner.partial_cmp(&other.inner)
    }
}

impl<'a, O, I> Ord for BoxOwned<'a, O, I, ByContent>
where
    O: ?Sized,
    I: Ord,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.inner.cmp(&other.inner)
    }
}

impl<'a, O, I> Hash for BoxOwned<'a, O, I, ByContent>
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

impl<'a, O, I> PartialEq<Self> for BoxOwned<'a, O, &'a mut I, ByAddress>
where
    O: ?Sized,
    I: ?Sized,
{
    fn eq(&self, other: &Self) -> bool {
        ptr::eq(self.inner as *const I, other.inner as *const I)
    }
}

impl<'a, O, I> Eq for BoxOwned<'a, O, &'a mut I, ByAddress>
where
    O: ?Sized,
    I: ?Sized,
{
}

impl<'a, O, I> PartialOrd<Self> for BoxOwned<'a, O, &'a mut I, ByAddress>
where
    O: ?Sized,
    I: ?Sized,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        (self.inner as *const I).partial_cmp(&(other.inner as *const I))
    }
}

impl<'a, O, I> Ord for BoxOwned<'a, O, &'a mut I, ByAddress>
where
    O: ?Sized,
    I: ?Sized,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        (self.inner as *const I).cmp(&(other.inner as *const I))
    }
}

impl<'a, O, I> Hash for BoxOwned<'a, O, &'a mut I, ByAddress>
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

impl<'a, O, I> PartialEq<Self> for BoxOwned<'a, O, &'a I, ByAddress>
where
    O: ?Sized,
    I: ?Sized,
{
    fn eq(&self, other: &Self) -> bool {
        ptr::eq(self.inner as *const I, other.inner as *const I)
    }
}

impl<'a, O, I> Eq for BoxOwned<'a, O, &'a I, ByAddress>
where
    O: ?Sized,
    I: ?Sized,
{
}

impl<'a, O, I> PartialOrd<Self> for BoxOwned<'a, O, &'a I, ByAddress>
where
    O: ?Sized,
    I: ?Sized,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        (self.inner as *const I).partial_cmp(&(other.inner as *const I))
    }
}

impl<'a, O, I> Ord for BoxOwned<'a, O, &'a I, ByAddress>
where
    O: ?Sized,
    I: ?Sized,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        (self.inner as *const I).cmp(&(other.inner as *const I))
    }
}

impl<'a, O, I> Hash for BoxOwned<'a, O, &'a I, ByAddress>
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

impl<'a, O, I, E> AsRef<I> for BoxOwned<'a, O, I, E>
where
    O: ?Sized,
    E: EqKind,
{
    fn as_ref(&self) -> &I {
        self.deref()
    }
}

impl<'a, O, I, E> AsMut<I> for BoxOwned<'a, O, I, E>
where
    O: ?Sized,
    E: EqKind,
{
    fn as_mut(&mut self) -> &mut I {
        self.deref_mut()
    }
}

impl<'a, O, I, E> Borrow<I> for BoxOwned<'a, O, I, E>
where
    O: ?Sized,
    E: EqKind,
{
    fn borrow(&self) -> &I {
        self.deref()
    }
}

impl<'a, O, I, E> Deref for BoxOwned<'a, O, I, E>
where
    O: ?Sized,
    E: EqKind,
{
    type Target = I;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'a, O, I, E> DerefMut for BoxOwned<'a, O, I, E>
where
    O: ?Sized,
    E: EqKind,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<'a, O, E> From<Box<O>> for BoxOwned<'a, O, &'a mut O, E>
where
    O: ?Sized,
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
