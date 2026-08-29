// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Unit tests for the stateful predicate module.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

use qubit_function::predicates::ArcStatefulPredicate;
use qubit_function::predicates::BoxStatefulPredicate;
use qubit_function::predicates::RcStatefulPredicate;
use qubit_function::predicates::StatefulPredicate;

#[test]
fn test_closure_implements_stateful_predicate() {
    let mut calls = 0;
    let mut predicate = |value: &i32| {
        calls += 1;
        calls % 2 == 0 && *value > 0
    };

    assert!(!predicate.test(&5));
    assert!(predicate.test(&5));
    assert!(!predicate.test(&-5));
}

#[test]
fn test_box_stateful_predicate_new_and_name_methods() {
    let mut predicate = BoxStatefulPredicate::new_with_name("positive", |value: &i32| *value > 0);

    assert_eq!(predicate.name(), Some("positive"));
    assert!(predicate.test(&5));
    assert!(!predicate.test(&-5));

    predicate.set_name("renamed");
    assert_eq!(predicate.name(), Some("renamed"));
}

#[test]
fn test_box_stateful_predicate_always_true_and_false() {
    let mut always_true = BoxStatefulPredicate::<i32>::always_true();
    let mut always_false = BoxStatefulPredicate::<i32>::always_false();

    assert_eq!(always_true.name(), Some("always_true"));
    assert_eq!(always_false.name(), Some("always_false"));
    assert!(always_true.test(&0));
    assert!(!always_false.test(&0));
}

#[test]
fn test_box_stateful_predicate_composition_preserves_state() {
    let left_calls = Rc::new(RefCell::new(0));
    let right_calls = Rc::new(RefCell::new(0));

    let left_counter = Rc::clone(&left_calls);
    let right_counter = Rc::clone(&right_calls);

    let left = BoxStatefulPredicate::new(move |value: &i32| {
        *left_counter.borrow_mut() += 1;
        *value > 0
    });
    let right = BoxStatefulPredicate::new(move |value: &i32| {
        *right_counter.borrow_mut() += 1;
        value % 2 == 0
    });

    let mut combined = left.and(right);

    assert!(combined.test(&4));
    assert!(!combined.test(&3));
    assert!(!combined.test(&-2));
    assert_eq!(*left_calls.borrow(), 3);
    assert_eq!(*right_calls.borrow(), 2);
}

#[test]
fn test_box_stateful_predicate_logical_operations_cover_all_branches_migrated() {
    let mut and_pred = BoxStatefulPredicate::new(|value: &i32| *value > 0).and(|value: &i32| value % 2 == 0);
    assert!(and_pred.test(&4));
    assert!(!and_pred.test(&3));
    assert!(!and_pred.test(&-2));

    let mut or_pred = BoxStatefulPredicate::new(|value: &i32| *value > 0).or(|value: &i32| value % 2 == 0);
    assert!(or_pred.test(&4));
    assert!(or_pred.test(&-2));
    assert!(!or_pred.test(&-3));

    let mut nand_pred = BoxStatefulPredicate::new(|value: &i32| *value > 0).nand(|value: &i32| value % 2 == 0);
    assert!(!nand_pred.test(&4));
    assert!(nand_pred.test(&3));
    assert!(nand_pred.test(&-2));

    let mut xor_pred = BoxStatefulPredicate::new(|value: &i32| *value > 0).xor(|value: &i32| value % 2 == 0);
    assert!(!xor_pred.test(&4));
    assert!(xor_pred.test(&3));
    assert!(xor_pred.test(&-2));
    assert!(!xor_pred.test(&-3));

    let mut nor_pred = BoxStatefulPredicate::new(|value: &i32| *value > 0).nor(|value: &i32| value % 2 == 0);
    assert!(!nor_pred.test(&4));
    assert!(!nor_pred.test(&-2));
    assert!(nor_pred.test(&-3));
}

#[test]
fn test_box_stateful_predicate_debug_and_display() {
    let unnamed = BoxStatefulPredicate::new(|value: &i32| *value > 0);
    assert!(format!("{unnamed:?}").contains("BoxStatefulPredicate"));
    assert_eq!(format!("{unnamed}"), "BoxStatefulPredicate(unnamed)");

    let named = BoxStatefulPredicate::new_with_name("positive", |value: &i32| *value > 0);
    assert!(format!("{named:?}").contains("positive"));
    assert_eq!(format!("{named}"), "BoxStatefulPredicate(positive)");
}

#[test]
fn test_rc_stateful_predicate_clone_shares_state() {
    let calls = Rc::new(RefCell::new(0));
    let counter = Rc::clone(&calls);
    let mut predicate = RcStatefulPredicate::new(move |value: &i32| {
        *counter.borrow_mut() += 1;
        *value > 0
    });
    let mut cloned = predicate.clone();

    assert!(predicate.test(&1));
    assert!(cloned.test(&2));
    assert_eq!(*calls.borrow(), 2);
}

#[test]
fn test_rc_stateful_predicate_logical_operations_cover_all_branches() {
    let positive = RcStatefulPredicate::new(|value: &i32| *value > 0);

    let mut and_pred = positive.and(|value: &i32| value % 2 == 0);
    assert!(and_pred.test(&4));
    assert!(!and_pred.test(&3));
    assert!(!and_pred.test(&-2));

    let mut or_pred = positive.or(|value: &i32| value % 2 == 0);
    assert!(or_pred.test(&4));
    assert!(or_pred.test(&-2));
    assert!(!or_pred.test(&-3));

    let mut nand_pred = positive.nand(|value: &i32| value % 2 == 0);
    assert!(!nand_pred.test(&4));
    assert!(nand_pred.test(&3));
    assert!(nand_pred.test(&-2));

    let mut xor_pred = positive.xor(|value: &i32| value % 2 == 0);
    assert!(!xor_pred.test(&4));
    assert!(xor_pred.test(&3));
    assert!(xor_pred.test(&-2));
    assert!(!xor_pred.test(&-3));

    let mut nor_pred = positive.nor(|value: &i32| value % 2 == 0);
    assert!(!nor_pred.test(&4));
    assert!(!nor_pred.test(&-2));
    assert!(nor_pred.test(&-3));
}

#[test]
fn test_rc_stateful_predicate_self_composition_releases_borrow_between_calls() {
    let calls = Rc::new(RefCell::new(0));
    let counter = Rc::clone(&calls);
    let predicate = RcStatefulPredicate::new(move |_value: &i32| {
        *counter.borrow_mut() += 1;
        true
    });
    let mut combined = predicate.and(predicate.clone());

    assert!(combined.test(&1));
    assert_eq!(*calls.borrow(), 2);
}

#[test]
fn test_arc_stateful_predicate_clone_shares_state() {
    let calls = Arc::new(AtomicUsize::new(0));
    let counter = Arc::clone(&calls);
    let mut predicate = ArcStatefulPredicate::new(move |value: &i32| {
        counter.fetch_add(1, Ordering::Relaxed);
        *value > 0
    });
    let mut cloned = predicate.clone();

    assert!(predicate.test(&1));
    assert!(cloned.test(&2));
    assert_eq!(calls.load(Ordering::Relaxed), 2);
}

#[test]
fn test_arc_stateful_predicate_logical_operations_cover_all_branches() {
    let positive = ArcStatefulPredicate::new(|value: &i32| *value > 0);

    let mut and_pred = positive.and(|value: &i32| value % 2 == 0);
    assert!(and_pred.test(&4));
    assert!(!and_pred.test(&3));
    assert!(!and_pred.test(&-2));

    let mut or_pred = positive.or(|value: &i32| value % 2 == 0);
    assert!(or_pred.test(&4));
    assert!(or_pred.test(&-2));
    assert!(!or_pred.test(&-3));

    let mut nand_pred = positive.nand(|value: &i32| value % 2 == 0);
    assert!(!nand_pred.test(&4));
    assert!(nand_pred.test(&3));
    assert!(nand_pred.test(&-2));

    let mut xor_pred = positive.xor(|value: &i32| value % 2 == 0);
    assert!(!xor_pred.test(&4));
    assert!(xor_pred.test(&3));
    assert!(xor_pred.test(&-2));
    assert!(!xor_pred.test(&-3));

    let mut nor_pred = positive.nor(|value: &i32| value % 2 == 0);
    assert!(!nor_pred.test(&4));
    assert!(!nor_pred.test(&-2));
    assert!(nor_pred.test(&-3));
}

#[test]
fn test_arc_stateful_predicate_self_composition_releases_lock_between_calls() {
    let calls = Arc::new(AtomicUsize::new(0));
    let counter = Arc::clone(&calls);
    let predicate = ArcStatefulPredicate::new(move |_value: &i32| {
        counter.fetch_add(1, Ordering::Relaxed);
        true
    });
    let mut combined = predicate.and(predicate.clone());

    assert!(combined.test(&1));
    assert_eq!(calls.load(Ordering::Relaxed), 2);
}

#[test]
fn test_stateful_predicate_not_operator() {
    let mut boxed = !BoxStatefulPredicate::new(|value: &i32| *value > 0);
    assert!(!boxed.test(&5));
    assert!(boxed.test(&-5));

    let rc = RcStatefulPredicate::new(|value: &i32| *value > 0);
    let mut negated_rc = !&rc;
    assert!(!negated_rc.test(&5));
    assert!(negated_rc.test(&-5));

    let mut owned_negated_rc = !rc;
    assert!(!owned_negated_rc.test(&5));
    assert!(owned_negated_rc.test(&-5));

    let arc = ArcStatefulPredicate::new(|value: &i32| *value > 0);
    let mut negated_arc = !&arc;
    assert!(!negated_arc.test(&5));
    assert!(negated_arc.test(&-5));

    let mut owned_negated_arc = !arc;
    assert!(!owned_negated_arc.test(&5));
    assert!(owned_negated_arc.test(&-5));
}

#[test]
fn test_box_stateful_predicate_observable_behavior() {
    let mut calls = 0;
    let predicate = BoxStatefulPredicate::new(move |value: &i32| {
        calls += 1;
        calls % 2 == 0 && *value > 0
    });

    let mut composed = predicate.or(|value: &i32| *value == 0);

    assert!(composed.test(&0));
    assert!(composed.test(&5));
    assert!(!composed.test(&-5));
}

#[test]
fn test_box_stateful_predicate_logical_operations_cover_all_branches() {
    let mut and_pred = BoxStatefulPredicate::new(|value: &i32| *value > 0).and(|value: &i32| value % 2 == 0);
    assert!(and_pred.test(&4));
    assert!(!and_pred.test(&3));
    assert!(!and_pred.test(&-2));

    let mut or_pred = BoxStatefulPredicate::new(|value: &i32| *value > 0).or(|value: &i32| value % 2 == 0);
    assert!(or_pred.test(&4));
    assert!(or_pred.test(&-2));
    assert!(!or_pred.test(&-3));

    let mut not_pred = !BoxStatefulPredicate::new(|value: &i32| *value > 0);
    assert!(!not_pred.test(&5));
    assert!(not_pred.test(&-5));

    let mut nand_pred = BoxStatefulPredicate::new(|value: &i32| *value > 0).nand(|value: &i32| value % 2 == 0);
    assert!(!nand_pred.test(&4));
    assert!(nand_pred.test(&3));
    assert!(nand_pred.test(&-2));

    let mut xor_pred = BoxStatefulPredicate::new(|value: &i32| *value > 0).xor(|value: &i32| value % 2 == 0);
    assert!(!xor_pred.test(&4));
    assert!(xor_pred.test(&3));
    assert!(xor_pred.test(&-2));
    assert!(!xor_pred.test(&-3));

    let mut nor_pred = BoxStatefulPredicate::new(|value: &i32| *value > 0).nor(|value: &i32| value % 2 == 0);
    assert!(!nor_pred.test(&4));
    assert!(!nor_pred.test(&-2));
    assert!(nor_pred.test(&-3));
}
