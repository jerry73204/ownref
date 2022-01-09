use crate::{arc_owned::ArcOwned, arc_ref::ArcRef, box_owned::BoxOwned, marker::*};
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

/// Content ordered reference to data within an owner in [Box].
pub type BoxRefC<'a, O, I = O> = BoxRef<'a, O, I, ByContent>;

/// Pointer address ordered reference to data within an owner in [Box].
pub type BoxRefA<'a, O, I = O> = BoxRef<'a, O, I, ByAddress>;

/// Content ordered reference to data within an [Any] + [Send] owner in [Box].
pub type BoxRefAnyC<'a, I> = BoxRef<'a, dyn Any + Send + 'static, I, ByContent>;

/// Pointer address ordered reference to data within an [Any] + [Send] owner in [Box].
pub type BoxRefAnyA<'a, I> = BoxRef<'a, dyn Any + Send + 'static, I, ByAddress>;

/// Content ordered reference to data within an [Any] owner in [Box].
pub type BoxRefAnyLocalC<'a, I> = BoxRef<'a, dyn Any + 'static, I, ByContent>;

/// Pointer address ordered reference to data within an [Any] owner in [Box].
pub type BoxRefAnyLocalA<'a, I> = BoxRef<'a, dyn Any + 'static, I, ByAddress>;

/// Reference to data within an owner in [Box].
pub struct BoxRef<'a, O, I, E>
where
    O: ?Sized,
    I: ?Sized,
    E: EqKind,
{
    // inner goes before owner so that inner drops before owner
    pub(crate) _phantom: PhantomData<E>,
    pub(crate) inner: &'a mut I,
    pub(crate) owner: Box<O>,
}

impl<'a, O, E> BoxRef<'a, O, O, E>
where
    O: ?Sized,
    E: EqKind,
{
    /// Build from boxed data.
    pub fn from_box(owner: Box<O>) -> Self {
        owner.into()
    }
}

impl<'a, O, E> BoxRef<'a, O, O, E>
where
    E: EqKind,
{
    /// Build from owner.
    pub fn new(owner: O) -> Self {
        Box::new(owner).into()
    }
}

impl<'a, O, I, E> BoxRef<'a, O, I, E>
where
    O: ?Sized,
    I: ?Sized,
    E: EqKind,
{
    /// Discard the inner reference and return boxed owner.
    pub fn into_box(from: BoxRef<'a, O, I, E>) -> Box<O> {
        let Self { owner, .. } = from;
        owner
    }

    /// Convert to [BoxOwned].
    pub fn into_box_owned(from: BoxRef<'a, O, I, E>) -> BoxOwned<'a, O, &mut I, E> {
        let Self { owner, inner, .. } = from;
        BoxOwned {
            owner,
            inner,
            _phantom: PhantomData,
        }
    }

    /// Convert to [ArcOwned] without re-allocation.
    pub fn into_arc_owned(from: BoxRef<'a, O, I, E>) -> ArcOwned<'a, O, &'a mut I, E> {
        let Self { owner, inner, .. } = from;
        ArcOwned {
            owner: owner.into(),
            inner,
            _phantom: PhantomData,
        }
    }

    /// Convert to [ArcRef] without re-allocation.
    pub fn into_arc_ref(from: BoxRef<'a, O, I, E>) -> ArcRef<'a, O, I, E> {
        let Self { owner, inner, .. } = from;
        ArcRef {
            owner: owner.into(),
            inner,
            _phantom: PhantomData,
        }
    }

    /// Reset the inner reference to the reference to owner.
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

    /// Get the reference to the owner.
    pub fn owner(this: &'a BoxRef<'a, O, I, E>) -> &'a O {
        &this.owner
    }

    /// Applies function `f` to inner reference.
    pub fn map<T, F>(self, f: F) -> BoxRef<'a, O, T, E>
    where
        F: FnOnce(&'a mut I) -> &'a mut T,
        T: ?Sized,
    {
        let Self { owner, inner, .. } = self;

        BoxRef {
            owner,
            inner: f(inner),
            _phantom: PhantomData,
        }
    }

    /// Applies fallible function `f` to inner reference.
    pub fn try_map<Ok, Err, F>(self, f: F) -> Result<BoxRef<'a, O, Ok, E>, Err>
    where
        F: FnOnce(&'a mut I) -> Result<&'a mut Ok, Err>,
        Ok: ?Sized,
    {
        let Self { owner, inner, .. } = self;

        Ok(BoxRef {
            owner,
            inner: f(inner)?,
            _phantom: PhantomData,
        })
    }

    /// Applies function `f` that returns optional reference to inner reference.
    pub fn filter_map<T, F>(self, f: F) -> Option<BoxRef<'a, O, T, E>>
    where
        F: FnOnce(&'a mut I) -> Option<&'a mut T>,
        T: ?Sized,
    {
        let Self { owner, inner, .. } = self;

        Some(BoxRef {
            owner,
            inner: f(inner)?,
            _phantom: PhantomData,
        })
    }
}

impl<'a, O, I, E> BoxRef<'a, O, I, E>
where
    E: EqKind,
{
    /// Discard the inner reference and return the owner.
    pub fn into_owner(from: BoxRef<'a, O, I, E>) -> O {
        let Self { owner, .. } = from;
        *owner
    }

    /// Change the owner type to [Any] + [Send] trait object.
    pub fn into_any_owner(from: BoxRef<'a, O, I, E>) -> BoxRef<'a, dyn Any + Send + 'static, I, E>
    where
        O: Send + 'static,
    {
        let Self { owner, inner, .. } = from;

        BoxRef {
            inner,
            owner,
            _phantom: PhantomData,
        }
    }

    /// Change the owner type to [Any] trait object.
    pub fn into_any_owner_local(from: BoxRef<'a, O, I, E>) -> BoxRef<'a, dyn Any + 'static, I, E>
    where
        O: 'static,
    {
        let Self { owner, inner, .. } = from;

        BoxRef {
            inner,
            owner,
            _phantom: PhantomData,
        }
    }
}

impl<'a, I, E> BoxRef<'a, dyn Any + Send + 'static, I, E>
where
    I: ?Sized,
    E: EqKind,
{
    /// Downcast the [Any] + [Send] trait object owner to concrete type.
    pub fn downcast_owner<O>(this: Self) -> Result<BoxRef<'a, O, I, E>, Self>
    where
        O: Send + 'static,
    {
        let Self { owner, inner, .. } = this;

        match owner.downcast() {
            Ok(owner) => Ok(BoxRef {
                owner,
                inner,
                _phantom: PhantomData,
            }),
            Err(owner) => Err(BoxRef {
                owner,
                inner,
                _phantom: PhantomData,
            }),
        }
    }
}

impl<'a, I, E> BoxRef<'a, dyn Any + 'static, I, E>
where
    I: ?Sized,
    E: EqKind,
{
    /// Downcast the [Any] trait object owner to concrete type.
    pub fn downcast_owner_local<O>(this: Self) -> Result<BoxRef<'a, O, I, E>, Self>
    where
        O: 'static,
    {
        let Self { owner, inner, .. } = this;

        match owner.downcast() {
            Ok(owner) => Ok(BoxRef {
                owner,
                inner,
                _phantom: PhantomData,
            }),
            Err(owner) => Err(BoxRef {
                owner,
                inner,
                _phantom: PhantomData,
            }),
        }
    }
}

impl<'a, O, I, E> Debug for BoxRef<'a, O, I, E>
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

impl<'a, O, I, E> Display for BoxRef<'a, O, I, E>
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

impl<'a, O, I> PartialEq<Self> for BoxRef<'a, O, I, ByContent>
where
    O: ?Sized,
    I: ?Sized,
    I: PartialEq<I>,
{
    fn eq(&self, other: &Self) -> bool {
        self.inner.eq(&other.inner)
    }
}

impl<'a, O, I> Eq for BoxRef<'a, O, I, ByContent>
where
    I: Eq,
    O: ?Sized,
    I: ?Sized,
{
}

impl<'a, O, I> PartialOrd<Self> for BoxRef<'a, O, I, ByContent>
where
    O: ?Sized,
    I: ?Sized,
    I: PartialOrd<I>,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        self.inner.partial_cmp(&other.inner)
    }
}

impl<'a, O, I> Ord for BoxRef<'a, O, I, ByContent>
where
    O: ?Sized,
    I: ?Sized,
    I: Ord,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.inner.cmp(&other.inner)
    }
}

impl<'a, O, I> Hash for BoxRef<'a, O, I, ByContent>
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

impl<'a, O, I> PartialEq<Self> for BoxRef<'a, O, I, ByAddress>
where
    O: ?Sized,
    I: ?Sized,
{
    fn eq(&self, other: &Self) -> bool {
        ptr::eq(self.inner as *const I, other.inner as *const I)
    }
}

impl<'a, O, I> Eq for BoxRef<'a, O, I, ByAddress>
where
    O: ?Sized,
    I: ?Sized,
{
}

impl<'a, O, I> PartialOrd<Self> for BoxRef<'a, O, I, ByAddress>
where
    O: ?Sized,
    I: ?Sized,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        (self.inner as *const I).partial_cmp(&(other.inner as *const I))
    }
}

impl<'a, O, I> Ord for BoxRef<'a, O, I, ByAddress>
where
    O: ?Sized,
    I: ?Sized,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        (self.inner as *const I).cmp(&(other.inner as *const I))
    }
}

impl<'a, O, I> Hash for BoxRef<'a, O, I, ByAddress>
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

impl<'a, O, I, E> AsRef<I> for BoxRef<'a, O, I, E>
where
    O: ?Sized,
    I: ?Sized,
    E: EqKind,
{
    fn as_ref(&self) -> &I {
        self.deref()
    }
}

impl<'a, O, I, E> AsMut<I> for BoxRef<'a, O, I, E>
where
    O: ?Sized,
    I: ?Sized,
    E: EqKind,
{
    fn as_mut(&mut self) -> &mut I {
        self.deref_mut()
    }
}

impl<'a, O, I, E> Borrow<I> for BoxRef<'a, O, I, E>
where
    O: ?Sized,
    I: ?Sized,
    E: EqKind,
{
    fn borrow(&self) -> &I {
        self.deref()
    }
}

impl<'a, O, I, E> Deref for BoxRef<'a, O, I, E>
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

impl<'a, O, I, E> DerefMut for BoxRef<'a, O, I, E>
where
    O: ?Sized,
    I: ?Sized,
    E: EqKind,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner
    }
}

impl<'a, O, E> From<Box<O>> for BoxRef<'a, O, O, E>
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
