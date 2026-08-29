// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
#![cfg(feature = "full")]

use std::cmp::Ordering;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use qubit_function::ArcCallable;
use qubit_function::ArcComparator;
use qubit_function::ArcRunnable;
use qubit_function::ArcStatefulBiPredicate;
use qubit_function::ArcStatefulTester;
use qubit_function::BoxComparator;
use qubit_function::BoxStatefulBiPredicate;
use qubit_function::BoxStatefulTester;
use qubit_function::Callable;
use qubit_function::RcCallable;
use qubit_function::RcComparator;
use qubit_function::RcRunnable;
use qubit_function::RcStatefulBiPredicate;
use qubit_function::RcStatefulTester;
use qubit_function::Runnable;
use qubit_function::StatefulBiPredicate;
use qubit_function::StatefulTester;

/// Collects Rust source files below `directory` in deterministic order.
fn collect_rust_source_files(directory: &Path, files: &mut Vec<PathBuf>) {
    let entries = fs::read_dir(directory).expect("source directory should be readable for contract checks");
    for entry in entries {
        let path = entry.expect("source directory entry should be readable").path();
        if path.is_dir() {
            collect_rust_source_files(&path, files);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            files.push(path);
        }
    }
    files.sort();
}

#[test]
fn all_concrete_callback_wrappers_have_must_use_contracts() {
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    collect_rust_source_files(&source_root, &mut files);
    let mut wrapper_count = 0;

    for file in files {
        let source = fs::read_to_string(&file).expect("Rust source file should be readable");
        let lines = source.lines().collect::<Vec<_>>();
        for (index, line) in lines.iter().enumerate() {
            let declaration = line.trim_start();
            if declaration.starts_with("pub struct Box")
                || declaration.starts_with("pub struct LocalBox")
                || declaration.starts_with("pub struct Arc")
                || declaration.starts_with("pub struct Rc")
            {
                wrapper_count += 1;
                assert!(index > 0, "wrapper declaration needs an attribute");
                assert_eq!(
                    lines[index - 1],
                    "#[must_use = \"callback wrappers do nothing unless stored or invoked\"]",
                    "missing must_use contract before {}:{}",
                    file.display(),
                    index + 1,
                );
            }
        }
    }

    assert_eq!(wrapper_count, 162, "unexpected callback wrapper inventory");
}

#[test]
fn comparator_into_fn_preserves_comparison() {
    let arc = ArcComparator::new(|left: &i32, right: &i32| left.cmp(right));
    let boxed = BoxComparator::new(|left: &i32, right: &i32| left.cmp(right));
    let rc = RcComparator::new(|left: &i32, right: &i32| left.cmp(right));

    assert_eq!(arc.into_fn()(&2, &1), Ordering::Greater);
    assert_eq!(boxed.into_fn()(&1, &1), Ordering::Equal);
    assert_eq!(rc.into_fn()(&1, &2), Ordering::Less);
}

fn assert_arc_truth_table(mut predicate: ArcStatefulBiPredicate<bool, bool>, expected: [bool; 4]) {
    assert_eq!(predicate.test(&false, &false), expected[0]);
    assert_eq!(predicate.test(&false, &true), expected[1]);
    assert_eq!(predicate.test(&true, &false), expected[2]);
    assert_eq!(predicate.test(&true, &true), expected[3]);
}

fn assert_rc_truth_table(mut predicate: RcStatefulBiPredicate<bool, bool>, expected: [bool; 4]) {
    assert_eq!(predicate.test(&false, &false), expected[0]);
    assert_eq!(predicate.test(&false, &true), expected[1]);
    assert_eq!(predicate.test(&true, &false), expected[2]);
    assert_eq!(predicate.test(&true, &true), expected[3]);
}

#[test]
fn arc_stateful_bi_predicate_covers_boolean_combinators() {
    assert_arc_truth_table(
        ArcStatefulBiPredicate::new(|left: &bool, _: &bool| *left).and(|_: &bool, right: &bool| *right),
        [false, false, false, true],
    );
    assert_arc_truth_table(
        ArcStatefulBiPredicate::new(|left: &bool, _: &bool| *left).or(|_: &bool, right: &bool| *right),
        [false, true, true, true],
    );
    assert_arc_truth_table(
        ArcStatefulBiPredicate::new(|left: &bool, _: &bool| *left).nand(|_: &bool, right: &bool| *right),
        [true, true, true, false],
    );
    assert_arc_truth_table(
        ArcStatefulBiPredicate::new(|left: &bool, _: &bool| *left).xor(|_: &bool, right: &bool| *right),
        [false, true, true, false],
    );
    assert_arc_truth_table(
        ArcStatefulBiPredicate::new(|left: &bool, _: &bool| *left).nor(|_: &bool, right: &bool| *right),
        [true, false, false, false],
    );

    let mut owned_not = !ArcStatefulBiPredicate::new(|_: &bool, _: &bool| true);
    let source = ArcStatefulBiPredicate::new(|_: &bool, _: &bool| false);
    let mut borrowed_not = !&source;
    assert!(!owned_not.test(&false, &false));
    assert!(borrowed_not.test(&false, &false));
}

#[test]
fn rc_stateful_bi_predicate_covers_boolean_combinators() {
    assert_rc_truth_table(
        RcStatefulBiPredicate::new(|left: &bool, _: &bool| *left).and(|_: &bool, right: &bool| *right),
        [false, false, false, true],
    );
    assert_rc_truth_table(
        RcStatefulBiPredicate::new(|left: &bool, _: &bool| *left).or(|_: &bool, right: &bool| *right),
        [false, true, true, true],
    );
    assert_rc_truth_table(
        RcStatefulBiPredicate::new(|left: &bool, _: &bool| *left).nand(|_: &bool, right: &bool| *right),
        [true, true, true, false],
    );
    assert_rc_truth_table(
        RcStatefulBiPredicate::new(|left: &bool, _: &bool| *left).xor(|_: &bool, right: &bool| *right),
        [false, true, true, false],
    );
    assert_rc_truth_table(
        RcStatefulBiPredicate::new(|left: &bool, _: &bool| *left).nor(|_: &bool, right: &bool| *right),
        [true, false, false, false],
    );

    let mut owned_not = !RcStatefulBiPredicate::new(|_: &bool, _: &bool| true);
    let source = RcStatefulBiPredicate::new(|_: &bool, _: &bool| false);
    let mut borrowed_not = !&source;
    assert!(!owned_not.test(&false, &false));
    assert!(borrowed_not.test(&false, &false));
}

#[test]
fn stateful_bi_predicate_constants_cover_all_owners() {
    let mut arc_true = ArcStatefulBiPredicate::<i32, i32>::always_true();
    let mut arc_false = ArcStatefulBiPredicate::<i32, i32>::always_false();
    let mut box_true = BoxStatefulBiPredicate::<i32, i32>::always_true();
    let mut box_false = BoxStatefulBiPredicate::<i32, i32>::always_false();
    let mut rc_true = RcStatefulBiPredicate::<i32, i32>::always_true();
    let mut rc_false = RcStatefulBiPredicate::<i32, i32>::always_false();

    assert!(arc_true.test(&1, &2));
    assert!(!arc_false.test(&1, &2));
    assert!(box_true.test(&1, &2));
    assert!(!box_false.test(&1, &2));
    assert!(rc_true.test(&1, &2));
    assert!(!rc_false.test(&1, &2));
}

#[test]
fn shared_tasks_clone_their_callback_state() {
    let arc_callable = ArcCallable::<i32, ()>::new(|| Ok(1));
    let rc_callable = RcCallable::<i32, ()>::new(|| Ok(2));
    let arc_runnable = ArcRunnable::<()>::new(|| Ok(()));
    let rc_runnable = RcRunnable::<()>::new(|| Ok(()));

    assert_eq!(arc_callable.clone().call(), Ok(1));
    assert_eq!(rc_callable.clone().call(), Ok(2));
    assert_eq!(arc_runnable.clone().run(), Ok(()));
    assert_eq!(rc_runnable.clone().run(), Ok(()));
}

#[test]
fn stateful_tester_not_covers_owned_and_borrowed_wrappers() {
    let mut arc_owned = !ArcStatefulTester::new(|| true);
    let arc_source = ArcStatefulTester::new(|| false);
    let mut arc_borrowed = !&arc_source;
    let mut box_owned = !BoxStatefulTester::new(|| true);
    let mut rc_owned = !RcStatefulTester::new(|| true);
    let rc_source = RcStatefulTester::new(|| false);
    let mut rc_borrowed = !&rc_source;

    assert!(!arc_owned.test());
    assert!(arc_borrowed.test());
    assert!(!box_owned.test());
    assert!(!rc_owned.test());
    assert!(rc_borrowed.test());
}
