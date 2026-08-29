// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

#![cfg(feature = "full")]

use qubit_function::ArcBiPredicate;
use qubit_function::ArcComparator;
use qubit_function::ArcFunction;
use qubit_function::ArcPredicate;
use qubit_function::ArcStatefulBiPredicate;
use qubit_function::ArcStatefulPredicate;
use qubit_function::ArcStatefulSupplier;
use qubit_function::ArcStatefulTester;
use qubit_function::ArcSupplier;
use qubit_function::ArcTester;
use qubit_function::BoxBiPredicate;
use qubit_function::BoxCallable;
use qubit_function::BoxCallableOnce;
use qubit_function::BoxComparator;
use qubit_function::BoxFunction;
use qubit_function::BoxPredicate;
use qubit_function::BoxRunnable;
use qubit_function::BoxRunnableOnce;
use qubit_function::BoxRunnableWith;
use qubit_function::BoxStatefulBiPredicate;
use qubit_function::BoxStatefulPredicate;
use qubit_function::BoxStatefulSupplier;
use qubit_function::BoxStatefulTester;
use qubit_function::BoxSupplier;
use qubit_function::BoxTester;
use qubit_function::LocalBoxRunnableOnce;
use qubit_function::RcBiPredicate;
use qubit_function::RcFunction;
use qubit_function::RcPredicate;
use qubit_function::RcStatefulBiPredicate;
use qubit_function::RcStatefulPredicate;
use qubit_function::RcStatefulSupplier;
use qubit_function::RcStatefulTester;
use qubit_function::RcSupplier;
use qubit_function::RcTester;
use qubit_function::StatefulSupplier;

/// Verifies that owned and shared final wrappers support chainable naming.
#[test]
fn test_with_name_sets_name_on_final_wrappers() {
    let boxed = BoxFunction::new(|value: &i32| *value).with_name("boxed");
    let rc = RcFunction::new(|value: &i32| *value).with_name("rc");
    let arc = ArcFunction::new(|value: &i32| *value).with_name("arc");
    let once = BoxCallableOnce::new(|| Ok::<i32, ()>(1)).with_name("once");

    assert_eq!(boxed.name(), Some("boxed"));
    assert_eq!(rc.name(), Some("rc"));
    assert_eq!(arc.name(), Some("arc"));
    assert_eq!(once.name(), Some("once"));
}

/// Verifies that chainable renaming changes only the selected shared clone.
#[test]
fn test_with_name_keeps_shared_clone_metadata_independent() {
    let original = ArcFunction::new_with_name("original", |value: &i32| *value);
    let renamed = original.clone().with_name("renamed");

    assert_eq!(original.name(), Some("original"));
    assert_eq!(renamed.name(), Some("renamed"));
}

/// Verifies the preserve-or-clear contract for Comparator transformations.
#[test]
fn test_comparator_name_propagation_contract() {
    let reversed = BoxComparator::new_with_name("ascending", |left: &i32, right: &i32| left.cmp(right)).reversed();
    assert_eq!(reversed.name(), Some("ascending"));

    let first = ArcComparator::new_with_name("primary", |_: &i32, _: &i32| std::cmp::Ordering::Equal);
    let combined = first.then_comparing(|left: &i32, right: &i32| left.cmp(right));
    assert_eq!(combined.name(), None);
}

/// Verifies logical negation preserves names while multi-source logic clears.
#[test]
fn test_tester_name_propagation_contract() {
    let boxed = !BoxTester::new_with_name("ready", || true);
    let rc_source = RcTester::new_with_name("ready", || true);
    let rc = !&rc_source;
    let arc_source = ArcTester::new_with_name("ready", || true);
    let arc = !&arc_source;

    assert_eq!(boxed.name(), Some("ready"));
    assert_eq!(rc.name(), Some("ready"));
    assert_eq!(arc.name(), Some("ready"));
    assert_eq!(arc_source.and(|| true).name(), None);

    let boxed_stateful = !BoxStatefulTester::new_with_name("ready", || true);
    let rc_stateful_source = RcStatefulTester::new_with_name("ready", || true);
    let rc_stateful = !&rc_stateful_source;
    let arc_stateful_source = ArcStatefulTester::new_with_name("ready", || true);
    let arc_stateful = !&arc_stateful_source;

    assert_eq!(boxed_stateful.name(), Some("ready"));
    assert_eq!(rc_stateful.name(), Some("ready"));
    assert_eq!(arc_stateful.name(), Some("ready"));
    assert_eq!(arc_stateful_source.and(|| true).name(), None);
}

/// Verifies negation preserves names for every predicate family and owner.
#[test]
fn test_predicate_not_preserves_name() {
    assert_eq!((!BoxPredicate::new_with_name("p", |_: &i32| true)).name(), Some("p"));
    let rc = RcPredicate::new_with_name("p", |_: &i32| true);
    let arc = ArcPredicate::new_with_name("p", |_: &i32| true);
    assert_eq!((!&rc).name(), Some("p"));
    assert_eq!((!&arc).name(), Some("p"));

    assert_eq!(
        (!BoxBiPredicate::new_with_name("p", |_: &i32, _: &i32| true)).name(),
        Some("p")
    );
    let rc = RcBiPredicate::new_with_name("p", |_: &i32, _: &i32| true);
    let arc = ArcBiPredicate::new_with_name("p", |_: &i32, _: &i32| true);
    assert_eq!((!&rc).name(), Some("p"));
    assert_eq!((!&arc).name(), Some("p"));

    assert_eq!(
        (!BoxStatefulPredicate::new_with_name("p", |_: &i32| true)).name(),
        Some("p")
    );
    let rc = RcStatefulPredicate::new_with_name("p", |_: &i32| true);
    let arc = ArcStatefulPredicate::new_with_name("p", |_: &i32| true);
    assert_eq!((!&rc).name(), Some("p"));
    assert_eq!((!&arc).name(), Some("p"));

    assert_eq!(
        (!BoxStatefulBiPredicate::new_with_name("p", |_: &i32, _: &i32| true,)).name(),
        Some("p")
    );
    let rc = RcStatefulBiPredicate::new_with_name("p", |_: &i32, _: &i32| true);
    let arc = ArcStatefulBiPredicate::new_with_name("p", |_: &i32, _: &i32| true);
    assert_eq!((!&rc).name(), Some("p"));
    assert_eq!((!&arc).name(), Some("p"));
}

/// Verifies supplier mapping preserves identity and multi-source operations
/// clear it.
#[test]
fn test_supplier_name_propagation_contract() {
    let boxed = BoxSupplier::new_with_name("source", || 1);
    assert_eq!(boxed.map(|value| value + 1).name(), Some("source"));

    let rc = RcSupplier::new_with_name("source", || 1);
    assert_eq!(rc.map(|value| value + 1).name(), Some("source"));
    assert_eq!(rc.filter(|_: &i32| true).name(), None);

    let arc = ArcSupplier::new_with_name("source", || 1);
    assert_eq!(arc.map(|value| value + 1).name(), Some("source"));
    assert_eq!(arc.zip(|| 2).name(), None);

    let mut boxed = BoxStatefulSupplier::new_with_name("source", || 1).memoize();
    assert_eq!(boxed.name(), Some("source"));
    assert_eq!(boxed.get(), 1);

    let rc_source = RcStatefulSupplier::new_with_name("source", || 1);
    let mut rc_memoized = rc_source.memoize();
    assert_eq!(rc_memoized.name(), Some("source"));
    assert_eq!(rc_memoized.get(), 1);

    let arc_source = ArcStatefulSupplier::new_with_name("source", || 1);
    let mut arc_memoized = arc_source.memoize();
    assert_eq!(arc_memoized.name(), Some("source"));
    assert_eq!(arc_memoized.get(), 1);
}

/// Verifies task mapping preserves names and sequencing clears them.
#[test]
fn test_task_name_propagation_contract() {
    let boxed = BoxCallable::new_with_name("load", || Ok::<i32, ()>(1));
    assert_eq!(boxed.map(|value| value + 1).name(), Some("load"));

    let boxed = BoxCallable::new_with_name("load", || Err::<i32, ()>(()));
    assert_eq!(boxed.map_err(|error| error).name(), Some("load"));

    let chained = BoxCallable::new_with_name("load", || Ok::<i32, ()>(1)).and_then(|value| Ok(value + 1));
    assert_eq!(chained.name(), None);

    let chained_once = BoxCallableOnce::new_with_name("load", || Ok::<i32, ()>(1)).and_then(|value| Ok(value + 1));
    assert_eq!(chained_once.name(), None);

    let runnable = BoxRunnable::new_with_name("first", || Ok::<(), ()>(())).and_then(|| Ok(()));
    assert_eq!(runnable.name(), None);

    let callable = BoxRunnable::new_with_name("first", || Ok::<(), ()>(())).then_callable(|| Ok::<i32, ()>(1));
    assert_eq!(callable.name(), None);

    let callable = BoxRunnableOnce::new_with_name("first", || Ok::<(), ()>(())).then_callable(|| Ok::<i32, ()>(1));
    assert_eq!(callable.name(), None);

    let callable = LocalBoxRunnableOnce::new_with_name("first", || Ok::<(), ()>(())).then_callable(|| Ok::<i32, ()>(1));
    assert_eq!(callable.name(), None);

    let callable = BoxRunnableWith::new_with_name("first", |_: &mut i32| Ok::<(), ()>(()))
        .then_callable_with(|value: &mut i32| Ok::<i32, ()>(*value));
    assert_eq!(callable.name(), None);
}
