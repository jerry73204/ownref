//! Marker types.

use std::marker::PhantomData;

/// Common trait for ordering behavior marker types.
pub trait EqKind {}

impl EqKind for ByAddress {}
impl EqKind for ByContent {}

/// Zero-sized type that marks ordering by pointer addresss.
pub struct ByAddress {
    _phandom: PhantomData<()>,
}

/// Zero-sized type that marks ordering by content.
pub struct ByContent {
    _phandom: PhantomData<()>,
}
