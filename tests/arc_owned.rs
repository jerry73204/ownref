use indexmap::IndexMap;
use ownref::{ArcOwnedA, ArcOwnedC};

#[test]
fn arc_owned_any_owner() {
    let x = ArcOwnedA::new(['a', 'b']);
    let x = x.map(|array| &array[0]);
    let x = ArcOwnedA::into_any_owner(x);
    let _: ArcOwnedA<[char; 2], _> = ArcOwnedA::downcast_owner(x)
        .map_err(|_| ())
        .expect("unable to downcast");
}

#[test]
fn arc_owned_a_iter() {
    let map: IndexMap<_, _> = [('a', 1), ('b', 2), ('c', 3)].into_iter().collect();
    let own: ArcOwnedA<IndexMap<char, usize>> = ArcOwnedA::new(map);

    let vec: Vec<ArcOwnedA<_, (&char, &usize)>> = own.clone().flatten().collect();
    assert_eq!(vec.len(), 3);
    assert_eq!(*vec[0], (&'a', &1));
    assert_eq!(*vec[1], (&'b', &2));
    assert_eq!(*vec[2], (&'c', &3));

    let keys: Vec<ArcOwnedA<_, &char>> = own.clone().flat_map(|map| map.keys()).collect();
    assert_eq!(keys.len(), 3);
    assert_eq!(*keys[0], &'a');
    assert_eq!(*keys[1], &'b');
    assert_eq!(*keys[2], &'c');

    let values: Vec<ArcOwnedA<_, &usize>> = own.flat_map(|map| map.values()).collect();
    assert_eq!(values.len(), 3);
    assert_eq!(*values[0], &1);
    assert_eq!(*values[1], &2);
    assert_eq!(*values[2], &3);
}

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
