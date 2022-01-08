use ownref::{BoxOwnedA, BoxOwnedC};

#[test]
fn box_owned_any_owner() {
    let x = BoxOwnedA::new(['a', 'b']);
    let x = x.map(|array| &array[0]);
    let x = BoxOwnedA::into_any_owner(x);
    let _: BoxOwnedA<[char; 2], _> = BoxOwnedA::downcast_owner(x)
        .map_err(|_| ())
        .expect("unable to downcast");
}

#[test]
fn box_owned_any_owner_local() {
    let x = BoxOwnedA::new(['a', 'b']);
    let x = x.map(|array| &array[0]);
    let x = BoxOwnedA::into_any_owner_local(x);
    let _: BoxOwnedA<[char; 2], _> = BoxOwnedA::downcast_owner_local(x)
        .map_err(|_| ())
        .expect("unable to downcast");
}

#[test]
fn box_owned_a() {
    let owner = BoxOwnedA::new(['a', 'b']);
    let _: &[char; 2] = &*owner;

    let ref_a: BoxOwnedA<[char; 2], &char> = owner.map(|array| &array[0]);
    assert_eq!(**ref_a, 'a');

    let owner: BoxOwnedA<[char; 2], &mut [char; 2]> = BoxOwnedA::into_owner_ref(ref_a);

    let ref_b: BoxOwnedA<[char; 2], &char> = owner.map(|array| &array[1]);
    assert_eq!(**ref_b, 'b');

    let array: [char; 2] = BoxOwnedA::into_owner(ref_b);
    assert_eq!(array, ['a', 'b']);
}

#[test]
fn box_owned_c() {
    let owner = BoxOwnedC::new(['a', 'b']);
    let _: &[char; 2] = &*owner;

    let ref_a: BoxOwnedC<[char; 2], &char> = owner.map(|array| &array[0]);
    assert_eq!(**ref_a, 'a');

    let owner: BoxOwnedC<[char; 2], &mut [char; 2]> = BoxOwnedC::into_owner_ref(ref_a);

    let ref_b: BoxOwnedC<[char; 2], &char> = owner.map(|array| &array[1]);
    assert_eq!(**ref_b, 'b');

    let array: [char; 2] = BoxOwnedC::into_owner(ref_b);
    assert_eq!(array, ['a', 'b']);
}
