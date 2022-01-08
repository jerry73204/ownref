use ownref::{ArcRefA, ArcRefC};

#[test]
fn arc_ref_any_owner() {
    let x = ArcRefA::new(['a', 'b']);
    let x = x.map(|array| &array[0]);
    let x = ArcRefA::into_any_owner(x);
    let _: ArcRefA<[char; 2], _> = ArcRefA::downcast_owner(x)
        .map_err(|_| ())
        .expect("unable to downcast");
}

#[test]
fn arc_ref_a_iter() {
    let own1 = ArcRefA::new(vec![3, 1, 4]);
    let refs: Vec<ArcRefA<Vec<usize>, usize>> = own1.flatten().collect();

    assert_eq!(ArcRefA::strong_count(&refs[0]), 3);
    assert_eq!(*refs[0], 3);
    assert_eq!(*refs[1], 1);
    assert_eq!(*refs[2], 4);
}

#[test]
fn arc_ref_a_cmp() {
    let own1 = ArcRefA::new(['a', 'a']);
    let own2 = own1.clone();
    assert_eq!(own1, own2);

    let ref1: ArcRefA<[char; 2], char> = own1.map(|array| &array[0]);
    let ref2: ArcRefA<[char; 2], char> = own2.map(|array| &array[1]);
    assert!(ref1 != ref2);
}

#[test]
fn arc_ref_c_cmp() {
    let own1 = ArcRefC::new(['a', 'a']);
    let own2 = own1.clone();
    assert_eq!(own1, own2);

    let ref1: ArcRefC<[char; 2], char> = own1.map(|array| &array[0]);
    let ref2: ArcRefC<[char; 2], char> = own2.map(|array| &array[1]);
    assert_eq!(ref1, ref2);
}

#[test]
fn arc_ref_a() {
    let owner = ArcRefA::new(['a', 'b']);
    let _: &[char; 2] = &*owner;

    let ref_a: ArcRefA<[char; 2], char> = owner.map(|array| &array[0]);
    assert_eq!(*ref_a, 'a');

    let owner: ArcRefA<[char; 2], [char; 2]> = ArcRefA::into_owner_ref(ref_a);

    let ref_b: ArcRefA<[char; 2], char> = owner.map(|array| &array[1]);
    assert_eq!(*ref_b, 'b');

    let array: [char; 2] = ArcRefA::unwrap_owner(ref_b);
    assert_eq!(array, ['a', 'b']);
}

#[test]
fn arc_ref_c() {
    let owner = ArcRefC::new(['a', 'b']);
    let _: &[char; 2] = &*owner;

    let ref_a: ArcRefC<[char; 2], char> = owner.map(|array| &array[0]);
    assert_eq!(*ref_a, 'a');

    let owner: ArcRefC<[char; 2], [char; 2]> = ArcRefC::into_owner_ref(ref_a);

    let ref_b: ArcRefC<[char; 2], char> = owner.map(|array| &array[1]);
    assert_eq!(*ref_b, 'b');

    let array: [char; 2] = ArcRefC::unwrap_owner(ref_b);
    assert_eq!(array, ['a', 'b']);
}
