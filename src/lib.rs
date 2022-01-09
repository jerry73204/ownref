//! References bundled with referee's owner, ordered by pointer address or data content.
//!
//! This crate provides the smart pointer type that bundles the data with its owner.
//! It has the folloing features:
//!
//! - The data is either a reference to a portion of the owner, or a data type that may
//!   contain references to the owner.
//! - The refernce can be ordered by data content or data pointer address.
//! - The owner is contained in [Box] or [Arc](std::sync::Arc).
//!
//! The following table shows [Box]-based reference types. The generic `O` denotes the
//! owner type and `I` denotes the data type.
//!
//! | data type (`I`) \\ ordering | Content ordered              | Pointer address ordered      |
//! |---------------------------- | ---------------------------- | ---------------------------- |
//! | Reference                   | [BoxRefC<O, I>](BoxRefC)     | [BoxRefA<O, I>](BoxRefA)     |
//! | Owned                       | [BoxOwnedC<O, I>](BoxOwnedC) | [BoxOwnedA<O, I>](BoxOwnedA) |
//!
//! The following table shows [Arc](std::sync::Arc)-based reference types.
//!
//! | data type (`I`) \\ ordering | Content ordered              | Pointer address ordered      |
//! |---------------------------- | ---------------------------- | ---------------------------- |
//! | Reference                   | [ArcRefC<O, I>](ArcRefC)     | [ArcRefA<O, I>](ArcRefA)     |
//! | Owned                       | [ArcOwnedC<O, I>](ArcOwnedC) | [ArcOwnedA<O, I>](ArcOwnedA) |
//!
//! For example,
//! - `BoxRefA<Vec<str>, str>` is a reference to `str` within the owner `Vec<str>`, which is ordered by pointer address.
//! - `ArcOwnedC<Vec<str>, Option<&str>>` stores the data type `Option<&str>`, which contains a reference within the owner `Vec<str>`.
//!   The reference is ordered by the data content.
//!
//! # Construction and destruction
//!
//! The smart references are built in the following ways.
//!
//! ```
//! # use std::sync::Arc;
//! use ownref::{ArcRefA, ArcRefC, BoxRefA, BoxRefC};
//!
//! struct Owner {
//!     a: u8,
//!     b: f32,
//! }
//!
//! // direct method
//! let _: BoxRefA<Owner, Owner> = BoxRefA::new(Owner { a: 7, b: 3.14 });
//! let _: ArcRefA<Owner, Owner> = ArcRefA::new(Owner { a: 7, b: 3.14 });
//!
//! // from boxed data
//! let boxed = Box::new(Owner { a: 7, b: 3.14 });
//! let _: BoxRefC<Owner, Owner> = BoxRefC::from(boxed);
//!
//! let boxed = Arc::new(Owner { a: 7, b: 3.14 });
//! let _: ArcRefC<Owner, Owner> = ArcRefC::from(boxed);
//! ```
//!
//! `BoxRef` is destructed by [BoxRef::into_owner()].
//!
//! ```
//! # use ownref::BoxRefA;
//! let owner = ['a', 'b'];
//! let boxref = BoxRefA::new(owner); // box the owner
//! let owner = BoxRefA::into_owner(boxref); // recover the owner
//! ```
//!
//! `ArcRef` is destructed by [ArcRef::unwrap_owner()]. It panics of the strong count is more than one.
//!
//! ```
//! # use ownref::ArcRefA;
//! let owner = ['a', 'b'];
//! let arcref = ArcRefA::new(owner); // box the owner
//! let owner = ArcRefA::unwrap_owner(arcref); // recover the owner
//! ```
//!
//! # Data type transformation
//!
//! The family of methods `map()`, `filter_map()` and `try_map()` can transform the data type.
//!
//! ```
//! # use ownref::{BoxRefA, BoxOwnedA};
//! struct Owner {
//!     a: u8,
//!     b: f32,
//! }
//!
//! let owner: BoxRefA<Owner, Owner> = BoxRefA::new(Owner { a: 7, b: 3.14 });
//! let inner: BoxRefA<Owner, f32> = owner.map(|data: &mut Owner| &mut data.b);
//!
//! // above is equivalent to
//! let owner: BoxOwnedA<Owner, &mut Owner> = BoxOwnedA::new(Owner { a: 7, b: 3.14 });
//! let inner: BoxOwnedA<Owner, &mut f32> = owner.map(|data: &mut Owner| &mut data.b);
//! ```
//!
//! Note that for `Owned` types. It is possible to transform the data such that the outcome
//! data does not reference to the owner.
//!
//! ```
//! # use ownref::{BoxOwnedA};
//! # struct Owner {
//! #     a: u8,
//! #     b: f32,
//! # }
//! let owner: BoxOwnedA<Owner, &mut Owner> = BoxOwnedA::new(Owner { a: 7, b: 3.14 });
//! let inner: BoxOwnedA<Owner, i32> = owner.map(|_| 5i32);
//! ```
//!
//! # Ordering
//!
//! [ArcRefA] is ordered by pointer address. The code below creates two references
//! pointing to distinct elements of an array, which element values are identical.
//! The references are distinguished by their pointer address.
//!
//! ```
//! # use ownref::ArcRefA;
//! let own1 = ArcRefA::new(['a', 'a']);
//! let own2 = own1.clone();
//!
//! let ref1: ArcRefA<[char; 2], char> = own1.map(|array| &array[0]);
//! let ref2: ArcRefA<[char; 2], char> = own2.map(|array| &array[1]);
//! assert!(ref1 != ref2); // differ by pointer address
//! ```
//!
//! [ArcRefC] is ordered by data content. The code below shows two references
//! pointing to distinct elements of an array, and are equalized by identical element value.
//!
//! ```
//! # use ownref::ArcRefC;
//! let own1 = ArcRefC::new(['a', 'a']);
//! let own2 = own1.clone();
//!
//! let ref1: ArcRefC<[char; 2], char> = own1.map(|array| &array[0]);
//! let ref2: ArcRefC<[char; 2], char> = own2.map(|array| &array[1]);
//! assert!(ref1 == ref2); // equalized by content
//! ```
//!
//! # Iterator flattening
//!
//! [ArcRef] is able to flatten the referenced data if the data type can be turned into an iterator.
//! For example, it can turn a reference to a vec into a vec of element references.
//!
//! ```
//! # use ownref::ArcRefC;
//! let vec: ArcRefC<Vec<char>> = ArcRefC::new(vec!['a', 'b', 'c']);
//! let collected: Vec<ArcRefC<Vec<char>, char>> = vec.flatten().collect();
//! ```
//!
//! [ArcOwned] is more versatile. It can flatten a map to references to its keys, values
//! or key-value pairs.
//!
//! ```
//! # use ownref::ArcOwnedC;
//! # use indexmap::IndexMap;
//! let map: IndexMap<_, _> = [('a', 1), ('b', 2), ('c', 3)].into_iter().collect();
//! let own: ArcOwnedC<_> = ArcOwnedC::new(map);
//!
//! let pairs: Vec<ArcOwnedC<_, (&char, &usize)>> =
//!     own.clone().flat_map(|map| map.iter()).collect();
//! let keys: Vec<ArcOwnedC<_, &char>> = own.clone().flat_map(|map| map.keys()).collect();
//! let values: Vec<ArcOwnedC<_, &usize>> = own.flat_map(|map| map.values()).collect();
//! ```
//!
//!
//! # Owner erasure
//!
//! The owner type can forget the owner type and keeps the data reference.
//!
//! ```
//! # use ownref::{BoxRefC, BoxRefAnyC};
//! let letter: BoxRefC<[char; 2], char> = BoxRefC::new(['a', 'b']).map(|array| &mut array[0]);
//!
//! // erase the owner
//! let letter: BoxRefAnyC<char> = BoxRefC::into_any_owner(letter);
//!
//! // recover the owner
//! let letter: BoxRefC<[char; 2], char> = match BoxRefC::downcast_owner(letter) {
//!     Ok(new_owner) => new_owner,
//!     Err(_old_owner) => unreachable!(),
//! };
//! ```
//!
//! This example creates two `u64` references within distinct owner types.
//! The owner type of references are erased so that they can be placed into a `Vec`.
//!
//! ```
//! # use ownref::{BoxRefAnyC, BoxRefC};
//! struct OwnerX {
//!     a: u8,
//!     b: u64,
//! }
//!
//! struct OwnerY {
//!     c: f32,
//!     d: u64,
//! }
//!
//! let own1 = BoxRefC::new(OwnerX { a: 7, b: 314 });
//! let ref1 = own1.map(|data| &mut data.b);
//!
//! let own2 = BoxRefC::new(OwnerY { c: 0.66, d: 52 });
//! let ref2 = own2.map(|data| &mut data.d);
//!
//! // ref1 and ref2 have different owners, erase ownwer types
//! let ref1: BoxRefAnyC<u64> = BoxRefC::into_any_owner(ref1);
//! let ref2: BoxRefAnyC<u64> = BoxRefC::into_any_owner(ref2);
//!
//! // so they can be stored in a vec
//! vec![ref1, ref2];
//! ```

mod arc_owned;
mod arc_ref;
mod box_owned;
mod box_ref;
pub mod marker;

pub use arc_owned::*;
pub use arc_ref::*;
pub use box_owned::*;
pub use box_ref::*;
