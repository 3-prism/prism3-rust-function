// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::thread;

use qubit_function::testers::stateful_tester::ArcStatefulTester;
use qubit_function::testers::stateful_tester::BoxStatefulTester;
use qubit_function::testers::stateful_tester::RcStatefulTester;
use qubit_function::testers::stateful_tester::StatefulTester;

#[derive(Clone)]
struct ThresholdTester {
    count: usize,
    threshold: usize,
}

impl StatefulTester for ThresholdTester {
    fn test(&mut self) -> bool {
        self.count += 1;
        self.count >= self.threshold
    }
}

#[test]
fn test_stateful_tester_closure_mutates_state() {
    let mut count = 0;
    let mut tester = move || {
        count += 1;
        count >= 2
    };

    assert!(!StatefulTester::test(&mut tester));
    assert!(StatefulTester::test(&mut tester));
}

/// Verifies that all stateful wrapper owners accept semantic tester objects.
#[test]
fn test_stateful_tester_combinators_accept_semantic_trait() {
    let mut boxed = BoxStatefulTester::new(|| true).and(ThresholdTester { count: 0, threshold: 1 });
    let mut rc = RcStatefulTester::new(|| true).and(ThresholdTester { count: 0, threshold: 1 });
    let mut arc = ArcStatefulTester::new(|| true).and(ThresholdTester { count: 0, threshold: 1 });

    assert!(boxed.test());
    assert!(rc.test());
    assert!(arc.test());
}

#[test]
fn test_box_stateful_tester_logical_operations_cover_branches() {
    let skipped = Rc::new(RefCell::new(false));
    let skipped_clone = Rc::clone(&skipped);
    let mut and_short = BoxStatefulTester::new(|| false).and(BoxStatefulTester::new(move || {
        *skipped_clone.borrow_mut() = true;
        true
    }));
    assert!(!and_short.test());
    assert!(!*skipped.borrow());

    let mut and_true = BoxStatefulTester::new(|| true).and(BoxStatefulTester::new(|| true));
    assert!(and_true.test());

    let skipped = Rc::new(RefCell::new(false));
    let skipped_clone = Rc::clone(&skipped);
    let mut or_short = BoxStatefulTester::new(|| true).or(BoxStatefulTester::new(move || {
        *skipped_clone.borrow_mut() = true;
        false
    }));
    assert!(or_short.test());
    assert!(!*skipped.borrow());

    let mut or_false = BoxStatefulTester::new(|| false).or(BoxStatefulTester::new(|| false));
    assert!(!or_false.test());

    let mut nand_false = BoxStatefulTester::new(|| true).nand(BoxStatefulTester::new(|| true));
    assert!(!nand_false.test());

    let mut nand_true = BoxStatefulTester::new(|| false).nand(BoxStatefulTester::new(|| true));
    assert!(nand_true.test());

    let mut xor_true = BoxStatefulTester::new(|| true).xor(BoxStatefulTester::new(|| false));
    assert!(xor_true.test());

    let mut xor_false = BoxStatefulTester::new(|| true).xor(BoxStatefulTester::new(|| true));
    assert!(!xor_false.test());

    let mut nor_true = BoxStatefulTester::new(|| false).nor(BoxStatefulTester::new(|| false));
    assert!(nor_true.test());

    let mut nor_false = BoxStatefulTester::new(|| true).nor(BoxStatefulTester::new(|| false));
    assert!(!nor_false.test());
}

#[test]
fn test_rc_stateful_tester_clone_shares_state() {
    let calls = Rc::new(RefCell::new(0));
    let calls_clone = Rc::clone(&calls);
    let mut tester = RcStatefulTester::new(move || {
        let mut calls = calls_clone.borrow_mut();
        *calls += 1;
        *calls >= 2
    });
    let mut clone = tester.clone();

    assert!(!tester.test());
    assert!(clone.test());
    assert_eq!(*calls.borrow(), 2);
}

#[test]
fn test_rc_stateful_tester_logical_operations_cover_branches() {
    let first = RcStatefulTester::new(|| false);
    let mut and_false = first.and(RcStatefulTester::new(|| true));
    assert!(!and_false.test());

    let first = RcStatefulTester::new(|| true);
    let mut and_true = first.and(RcStatefulTester::new(|| true));
    assert!(and_true.test());

    let first = RcStatefulTester::new(|| true);
    let mut or_true = first.or(RcStatefulTester::new(|| false));
    assert!(or_true.test());

    let first = RcStatefulTester::new(|| false);
    let mut or_false = first.or(RcStatefulTester::new(|| false));
    assert!(!or_false.test());

    let first = RcStatefulTester::new(|| true);
    let mut nand_false = first.nand(RcStatefulTester::new(|| true));
    assert!(!nand_false.test());

    let first = RcStatefulTester::new(|| false);
    let mut nand_true = first.nand(RcStatefulTester::new(|| true));
    assert!(nand_true.test());

    let first = RcStatefulTester::new(|| true);
    let mut xor_true = first.xor(RcStatefulTester::new(|| false));
    assert!(xor_true.test());

    let first = RcStatefulTester::new(|| true);
    let mut xor_false = first.xor(RcStatefulTester::new(|| true));
    assert!(!xor_false.test());

    let first = RcStatefulTester::new(|| false);
    let mut nor_true = first.nor(RcStatefulTester::new(|| false));
    assert!(nor_true.test());

    let first = RcStatefulTester::new(|| true);
    let mut nor_false = first.nor(RcStatefulTester::new(|| false));
    assert!(!nor_false.test());
}

#[test]
fn test_arc_stateful_tester_clone_shares_state_across_threads() {
    let calls = Arc::new(AtomicUsize::new(0));
    let calls_clone = Arc::clone(&calls);
    let mut tester = ArcStatefulTester::new(move || {
        calls_clone.fetch_add(1, Ordering::SeqCst);
        true
    });
    let clone = tester.clone();

    assert!(tester.test());
    let handle = thread::spawn(move || {
        let mut clone = clone;
        clone.test()
    });
    assert!(handle.join().expect("thread should not panic"));
    assert_eq!(calls.load(Ordering::SeqCst), 2);
}

#[test]
fn test_arc_stateful_tester_logical_operations_cover_branches() {
    let first = ArcStatefulTester::new(|| false);
    let mut and_false = first.and(ArcStatefulTester::new(|| true));
    assert!(!and_false.test());

    let first = ArcStatefulTester::new(|| true);
    let mut and_true = first.and(ArcStatefulTester::new(|| true));
    assert!(and_true.test());

    let first = ArcStatefulTester::new(|| true);
    let mut or_true = first.or(ArcStatefulTester::new(|| false));
    assert!(or_true.test());

    let first = ArcStatefulTester::new(|| false);
    let mut or_false = first.or(ArcStatefulTester::new(|| false));
    assert!(!or_false.test());

    let first = ArcStatefulTester::new(|| true);
    let mut nand_false = first.nand(ArcStatefulTester::new(|| true));
    assert!(!nand_false.test());

    let first = ArcStatefulTester::new(|| false);
    let mut nand_true = first.nand(ArcStatefulTester::new(|| true));
    assert!(nand_true.test());

    let first = ArcStatefulTester::new(|| true);
    let mut xor_true = first.xor(ArcStatefulTester::new(|| false));
    assert!(xor_true.test());

    let first = ArcStatefulTester::new(|| true);
    let mut xor_false = first.xor(ArcStatefulTester::new(|| true));
    assert!(!xor_false.test());

    let first = ArcStatefulTester::new(|| false);
    let mut nor_true = first.nor(ArcStatefulTester::new(|| false));
    assert!(nor_true.test());

    let first = ArcStatefulTester::new(|| true);
    let mut nor_false = first.nor(ArcStatefulTester::new(|| false));
    assert!(!nor_false.test());
}

#[test]
fn test_box_stateful_tester_logical_operations_cover_all_branches() {
    let mut and_true = BoxStatefulTester::new(|| true).and(|| true);
    assert!(and_true.test());

    let mut and_false = BoxStatefulTester::new(|| false).and(|| true);
    assert!(!and_false.test());

    let mut or_true = BoxStatefulTester::new(|| true).or(|| false);
    assert!(or_true.test());

    let mut or_false = BoxStatefulTester::new(|| false).or(|| false);
    assert!(!or_false.test());

    let mut not_true = !BoxStatefulTester::new(|| false);
    assert!(not_true.test());

    let mut nand_false = BoxStatefulTester::new(|| true).nand(|| true);
    assert!(!nand_false.test());

    let mut nand_true = BoxStatefulTester::new(|| false).nand(|| true);
    assert!(nand_true.test());

    let mut xor_true = BoxStatefulTester::new(|| true).xor(|| false);
    assert!(xor_true.test());

    let mut xor_false = BoxStatefulTester::new(|| true).xor(|| true);
    assert!(!xor_false.test());

    let mut nor_true = BoxStatefulTester::new(|| false).nor(|| false);
    assert!(nor_true.test());

    let mut nor_false = BoxStatefulTester::new(|| true).nor(|| false);
    assert!(!nor_false.test());
}

/// Verifies naming and diagnostic formatting for an owned stateful tester.
#[test]
fn test_box_stateful_tester_name_and_diagnostics() {
    let mut tester = BoxStatefulTester::new_with_name("threshold", || true);

    assert_eq!(tester.name(), Some("threshold"));
    assert_eq!(format!("{tester:?}"), "BoxStatefulTester { name: Some(\"threshold\") }");
    assert_eq!(format!("{tester}"), "BoxStatefulTester(threshold)");
    tester.clear_name();
    assert_eq!(tester.name(), None);
    assert_eq!(format!("{tester}"), "BoxStatefulTester");
    assert!(<BoxStatefulTester as StatefulTester>::test(&mut tester));
}

/// Verifies clone-independent metadata for a shared Rc stateful tester.
#[test]
fn test_rc_stateful_tester_name_and_diagnostics() {
    let original = RcStatefulTester::new_with_name("threshold", || true);
    let renamed = original.clone().with_name("renamed");

    assert_eq!(original.name(), Some("threshold"));
    assert_eq!(renamed.name(), Some("renamed"));
    assert_eq!(
        format!("{original:?}"),
        "RcStatefulTester { name: Some(\"threshold\") }"
    );
    assert_eq!(format!("{original}"), "RcStatefulTester(threshold)");
}

/// Verifies clone-independent metadata for a shared Arc stateful tester.
#[test]
fn test_arc_stateful_tester_name_and_diagnostics() {
    let original = ArcStatefulTester::new_with_optional_name(|| true, Some("threshold".to_owned()));
    let renamed = original.clone().with_name("renamed");

    assert_eq!(original.name(), Some("threshold"));
    assert_eq!(renamed.name(), Some("renamed"));
    assert_eq!(
        format!("{original:?}"),
        "ArcStatefulTester { name: Some(\"threshold\") }"
    );
    assert_eq!(format!("{original}"), "ArcStatefulTester(threshold)");
}
