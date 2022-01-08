use ownref::{ArcOwnedA, ArcOwnedC};

#[test]
fn arc_owned_a_cmp() {
    let own1 = ArcOwnedA::new(['a', 'a']);
    let own2 = own1.clone();
    assert_eq!(own1, own2);

    let ref1: ArcOwnedA<[char; 2], &char> = own1.map(|array| &array[0]);
    let ref2: ArcOwnedA<[char; 2], &char> = own2.map(|array| &array[1]);
    assert!(ref1 != ref2);
}

#[test]
fn arc_owned_c_cmp() {
    let own1 = ArcOwnedC::new(['a', 'a']);
    let own2 = own1.clone();
    assert_eq!(own1, own2);

    let ref1: ArcOwnedC<[char; 2], &char> = own1.map(|array| &array[0]);
    let ref2: ArcOwnedC<[char; 2], &char> = own2.map(|array| &array[1]);
    assert_eq!(ref1, ref2);
}

#[test]
fn arc_owned_a() {
    let owner = ArcOwnedA::new(['a', 'b']);
    let _: &[char; 2] = &*owner;

    let ref_a: ArcOwnedA<[char; 2], &char> = owner.map(|array| &array[0]);
    assert_eq!(**ref_a, 'a');

    let owner: ArcOwnedA<[char; 2], &[char; 2]> = ArcOwnedA::into_owner_ref(ref_a);

    let ref_b: ArcOwnedA<[char; 2], &char> = owner.map(|array| &array[1]);
    assert_eq!(**ref_b, 'b');

    let array: [char; 2] = ArcOwnedA::unwrap_owner(ref_b);
    assert_eq!(array, ['a', 'b']);
}

#[test]
fn arc_owned_c() {
    let owner = ArcOwnedC::new(['a', 'b']);
    let _: &[char; 2] = &*owner;

    let ref_a: ArcOwnedC<[char; 2], &char> = owner.map(|array| &array[0]);
    assert_eq!(**ref_a, 'a');

    let owner: ArcOwnedC<[char; 2], &[char; 2]> = ArcOwnedC::into_owner_ref(ref_a);

    let ref_b: ArcOwnedC<[char; 2], &char> = owner.map(|array| &array[1]);
    assert_eq!(**ref_b, 'b');

    let array: [char; 2] = ArcOwnedC::unwrap_owner(ref_b);
    assert_eq!(array, ['a', 'b']);
}
