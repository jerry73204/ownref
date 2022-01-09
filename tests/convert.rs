use ownref::{ArcOwnedA, ArcRefA, BoxOwnedA, BoxRefA};
use std::sync::Arc;

#[test]
fn box_convert() {
    let text = "a string";
    let boxed: Box<str> = String::from(text).into_boxed_str();

    let own: BoxOwnedA<str> = BoxOwnedA::from(boxed);
    assert_eq!(*own, text);

    let ref_: BoxRefA<str> = BoxOwnedA::into_box_ref(own);
    assert_eq!(&*ref_, text);

    let own2: BoxOwnedA<str> = BoxRefA::into_box_owned(ref_);
    assert_eq!(*own2, text);
}

#[test]
fn arc_convert() {
    let text: Arc<str> = String::from("a string").into_boxed_str().into();
    let own: ArcOwnedA<str> = ArcOwnedA::from(text);
    let ref_: ArcRefA<str> = ArcOwnedA::into_arc_ref(own.clone());
    let own2: ArcOwnedA<str> = ArcRefA::into_arc_owned(ref_.clone());

    assert_eq!(**own, *ref_);
    assert_eq!(**own2, *ref_);
    assert_eq!(**own, **own2);
}
