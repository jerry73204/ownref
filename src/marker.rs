use std::marker::PhantomData;

pub trait EqKind {}

impl EqKind for ByAddress {}
impl EqKind for ByContent {}

pub struct ByAddress {
    _phandom: PhantomData<()>,
}

pub struct ByContent {
    _phandom: PhantomData<()>,
}
