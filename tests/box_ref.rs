use ownref::{BoxRefA, BoxRefC};

#[test]
fn box_ref_any_owner() {
    let x = BoxRefA::new(['a', 'b']);
    let x = x.map(|array| &mut array[0]);
    let x = BoxRefA::into_any_owner(x);
    let _: BoxRefA<[char; 2], _> = BoxRefA::downcast_owner(x)
        .map_err(|_| ())
        .expect("unable to downcast");
}

#[test]
fn box_ref_any_owner_local() {
    let x = BoxRefA::new(['a', 'b']);
    let x = x.map(|array| &mut array[0]);
    let x = BoxRefA::into_any_owner_local(x);
    let _: BoxRefA<[char; 2], _> = BoxRefA::downcast_owner_local(x)
        .map_err(|_| ())
        .expect("unable to downcast");
}

#[test]
fn box_ref_a() {
    let owner = BoxRefA::new(['a', 'b']);
    let _: &[char; 2] = &*owner;

    let ref_a: BoxRefA<[char; 2], char> = owner.map(|array| &mut array[0]);
    assert_eq!(*ref_a, 'a');

    let owner: BoxRefA<[char; 2], [char; 2]> = BoxRefA::into_owner_ref(ref_a);

    let ref_b: BoxRefA<[char; 2], char> = owner.map(|array| &mut array[1]);
    assert_eq!(*ref_b, 'b');

    let array: [char; 2] = BoxRefA::into_owner(ref_b);
    assert_eq!(array, ['a', 'b']);
}

#[test]
fn box_ref_c() {
    let owner = BoxRefC::new(['a', 'b']);
    let _: &[char; 2] = &*owner;

    let ref_a: BoxRefC<[char; 2], char> = owner.map(|array| &mut array[0]);
    assert_eq!(*ref_a, 'a');

    let owner: BoxRefC<[char; 2], [char; 2]> = BoxRefC::into_owner_ref(ref_a);

    let ref_b: BoxRefC<[char; 2], char> = owner.map(|array| &mut array[1]);
    assert_eq!(*ref_b, 'b');

    let array: [char; 2] = BoxRefC::into_owner(ref_b);
    assert_eq!(array, ['a', 'b']);
}
