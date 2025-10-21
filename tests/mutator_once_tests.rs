/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # MutatorOnce Tests
//!
//! Tests the complete functionality of MutatorOnce trait and its implementations.

use prism3_function::{BoxMutatorOnce, MutatorOnce};

// Test closures specialization and default behaviors
#[test]
fn test_closure_into_and_to_variants() {
    let data = vec![1, 2, 3];
    let closure = move |x: &mut Vec<i32>| x.extend(data);

    // into_box consumes the closure and returns BoxMutatorOnce
    let boxed = closure.into_box();
    let mut v = vec![0];
    boxed.mutate(&mut v);
    assert_eq!(v, vec![0, 1, 2, 3]);

    // Note: closure was moved - create another closure for to_box/to_fn
    let closure2 = move |x: &mut Vec<i32>| x.push(99);
    // to_box uses Clone; simple closure is zero-sized and Clone, so to_box exists
    let boxed2 = closure2.to_box();
    let mut v2 = vec![0];
    boxed2.mutate(&mut v2);
    assert_eq!(v2, vec![0, 99]);

    // to_fn for cloneable closure
    let closure3 = move |x: &mut Vec<i32>| x.push(7);
    let f = closure3.to_fn();
    let mut v3 = vec![0];
    f(&mut v3);
    assert_eq!(v3, vec![0, 7]);
}

#[test]
fn test_box_mutator_once_identity_and_chain() {
    // identity: into_box should be identity for BoxMutatorOnce
    let m = BoxMutatorOnce::new(|x: &mut Vec<i32>| x.push(1));
    let m2 = m.into_box();
    let mut v = Vec::new();
    m2.mutate(&mut v);
    assert_eq!(v, vec![1]);

    // chain
    let m1 = BoxMutatorOnce::new(|x: &mut Vec<i32>| x.push(2));
    let m2 = BoxMutatorOnce::new(|x: &mut Vec<i32>| x.push(3));
    let chained = m1.and_then(m2);
    let mut v2 = Vec::new();
    chained.mutate(&mut v2);
    assert_eq!(v2, vec![2, 3]);
}

// Custom MutatorOnce using default into_box/into_fn/to_box/to_fn
struct MyMutatorOnce {
    data: Vec<i32>,
}

impl MutatorOnce<Vec<i32>> for MyMutatorOnce {
    fn mutate(self, value: &mut Vec<i32>) {
        value.extend(self.data);
    }
}

#[test]
fn test_custom_mutator_default_adapters() {
    let my = MyMutatorOnce { data: vec![4, 5] };
    let boxed = my.into_box();
    let mut v = vec![0];
    boxed.mutate(&mut v);
    assert_eq!(v, vec![0, 4, 5]);

    // to test to_box/to_fn we need a cloneable type
    #[derive(Clone)]
    struct CloneMutator {
        data: Vec<i32>,
    }
    impl MutatorOnce<Vec<i32>> for CloneMutator {
        fn mutate(self, value: &mut Vec<i32>) {
            value.extend(self.data);
        }
    }

    let c = CloneMutator { data: vec![6] };
    let boxed_c = c.to_box();
    let mut v2 = vec![0];
    boxed_c.mutate(&mut v2);
    assert_eq!(v2, vec![0, 6]);

    let c2 = CloneMutator { data: vec![8] };
    let f = c2.to_fn();
    let mut v3 = vec![0];
    f(&mut v3);
    assert_eq!(v3, vec![0, 8]);
}
