# ownref

\[ [doc](https://docs.rs/ownref/) | [crates.io](https://crates.io/crates/ownref) \]

This crate provides the smart pointer type that bundles the data with its owner.
It has the folloing features:

- The data is either a reference to a portion of the owner, or a data type that may
  contain references to the owner.
- The refernce can be ordered by data content or data pointer address.
- The owner is contained in `Box` or `Arc`.

The following table shows `Box`-based reference types. The generic `O` denotes the
owner type and `I` denotes the data type.

| data type (`I`) \\ ordering | Content ordered              | Pointer address ordered      |
|---------------------------- | ---------------------------- | ---------------------------- |
| Reference                   | `BoxRefC<O, I>`     | `BoxRefA<O, I>`     |
| Owned                       | `BoxOwnedC<O, I>` | `BoxOwnedA<O, I>` |

The following table shows `Arc`-based reference types.

| data type (`I`) \\ ordering | Content ordered              | Pointer address ordered      |
|---------------------------- | ---------------------------- | ---------------------------- |
| Reference                   | `ArcRefC<O, I>`     | `ArcRefA<O, I>`     |
| Owned                       | `ArcOwnedC<O, I>` | `ArcOwnedA<O, I>` |

For example,
- `BoxRefA<Vec<str>, str>` is a reference to `str` within the owner `Vec<str>`, which is ordered by pointer address.
- `ArcOwnedC<Vec<str>, Option<&str>>` stores the data type `Option<&str>`, which contains a reference within the owner `Vec<str>`.
  The reference is ordered by the data content.

# License

MIT license. See [license file](LICENSE.txt).
