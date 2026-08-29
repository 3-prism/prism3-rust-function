// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::cell::Cell;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

use qubit_function::predicates::ArcStatefulBiPredicate;
use qubit_function::predicates::BoxStatefulBiPredicate;
use qubit_function::predicates::RcStatefulBiPredicate;
use qubit_function::predicates::StatefulBiPredicate;

fn box_stateful_bi_predicate_returning(value: bool) -> BoxStatefulBiPredicate<i32, i32> {
    BoxStatefulBiPredicate::new(move |_: &i32, _: &i32| value)
}

#[test]
fn test_box_stateful_bi_predicate_tracks_state_and_short_circuits() {
    let mut calls = 0;
    let mut predicate =
        BoxStatefulBiPredicate::new_with_name("every_second_positive_sum", move |first: &i32, second: &i32| {
            calls += 1;
            calls % 2 == 0 && first + second > 0
        });

    assert_eq!(predicate.name(), Some("every_second_positive_sum"));
    assert!(!predicate.test(&5, &3));
    assert!(predicate.test(&5, &3));

    let rhs_calls = Rc::new(Cell::new(0));
    let rhs_seen = rhs_calls.clone();
    let mut and_predicate = BoxStatefulBiPredicate::new(|_: &i32, _: &i32| false).and(BoxStatefulBiPredicate::new(
        move |_: &i32, _: &i32| {
            rhs_seen.set(rhs_seen.get() + 1);
            true
        },
    ));

    assert!(!and_predicate.test(&1, &2));
    assert_eq!(rhs_calls.get(), 0);

    let rhs_calls = Rc::new(Cell::new(0));
    let rhs_seen = rhs_calls.clone();
    let mut or_predicate = BoxStatefulBiPredicate::new(|_: &i32, _: &i32| true).or(BoxStatefulBiPredicate::new(
        move |_: &i32, _: &i32| {
            rhs_seen.set(rhs_seen.get() + 1);
            false
        },
    ));

    assert!(or_predicate.test(&1, &2));
    assert_eq!(rhs_calls.get(), 0);
}

#[test]
fn test_rc_stateful_bi_predicate_clones_share_state_and_can_be_negated() {
    let log = Rc::new(RefCell::new(Vec::new()));
    let observed = log.clone();
    let mut predicate = RcStatefulBiPredicate::new(move |first: &i32, second: &i32| {
        observed.borrow_mut().push(first + second);
        first > second
    });
    let mut clone = predicate.clone();

    assert!(predicate.test(&5, &3));
    assert!(!clone.test(&2, &7));
    assert_eq!(*log.borrow(), vec![8, 9]);

    let mut negated = !&predicate;
    assert!(!negated.test(&9, &1));
}

#[test]
fn test_arc_stateful_bi_predicate_can_be_shared_across_threads() {
    let log = Arc::new(Mutex::new(Vec::new()));
    let observed = log.clone();
    let predicate = ArcStatefulBiPredicate::new(move |first: &i32, second: &i32| {
        let sum = first + second;
        observed.lock().expect("mutex should not be poisoned").push(sum);
        sum > 0
    });

    let mut thread_predicate = predicate.clone();
    let handle = thread::spawn(move || thread_predicate.test(&5, &3));
    assert!(handle.join().expect("thread should not panic"));

    let mut local_predicate = predicate.clone();
    assert!(!local_predicate.test(&-10, &2));
    assert_eq!(*log.lock().expect("mutex should not be poisoned"), vec![8, -8]);
}

#[test]
fn test_box_stateful_bi_predicate_logical_methods() {
    let mut nand =
        BoxStatefulBiPredicate::new(|_: &i32, _: &i32| true).nand(BoxStatefulBiPredicate::new(|_: &i32, _: &i32| true));
    assert!(!nand.test(&1, &2));

    let mut xor =
        BoxStatefulBiPredicate::new(|_: &i32, _: &i32| true).xor(BoxStatefulBiPredicate::new(|_: &i32, _: &i32| false));
    assert!(xor.test(&1, &2));

    let mut nor = BoxStatefulBiPredicate::new(|_: &i32, _: &i32| false)
        .nor(BoxStatefulBiPredicate::new(|_: &i32, _: &i32| false));
    assert!(nor.test(&1, &2));

    let mut negated = !BoxStatefulBiPredicate::new(|_: &i32, _: &i32| true);
    assert!(!negated.test(&1, &2));
}

#[test]
fn test_box_stateful_bi_predicate_logical_truth_tables() {
    for (left, right, expected) in [
        (true, true, true),
        (true, false, false),
        (false, true, false),
        (false, false, false),
    ] {
        let mut predicate = box_stateful_bi_predicate_returning(left).and(box_stateful_bi_predicate_returning(right));
        assert_eq!(predicate.test(&1, &2), expected);
    }

    for (left, right, expected) in [
        (true, true, true),
        (true, false, true),
        (false, true, true),
        (false, false, false),
    ] {
        let mut predicate = box_stateful_bi_predicate_returning(left).or(box_stateful_bi_predicate_returning(right));
        assert_eq!(predicate.test(&1, &2), expected);
    }

    for (left, right, expected) in [
        (true, true, false),
        (true, false, true),
        (false, true, true),
        (false, false, true),
    ] {
        let mut predicate = box_stateful_bi_predicate_returning(left).nand(box_stateful_bi_predicate_returning(right));
        assert_eq!(predicate.test(&1, &2), expected);
    }

    for (left, right, expected) in [
        (true, true, false),
        (true, false, true),
        (false, true, true),
        (false, false, false),
    ] {
        let mut predicate = box_stateful_bi_predicate_returning(left).xor(box_stateful_bi_predicate_returning(right));
        assert_eq!(predicate.test(&1, &2), expected);
    }

    for (left, right, expected) in [
        (true, true, false),
        (true, false, false),
        (false, true, false),
        (false, false, true),
    ] {
        let mut predicate = box_stateful_bi_predicate_returning(left).nor(box_stateful_bi_predicate_returning(right));
        assert_eq!(predicate.test(&1, &2), expected);
    }

    let mut negated = !box_stateful_bi_predicate_returning(false);
    assert!(negated.test(&1, &2));
}
