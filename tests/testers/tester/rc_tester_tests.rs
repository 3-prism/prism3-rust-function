// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_function::testers::tester::RcTester;
use qubit_function::testers::tester::Tester;

struct RcAlwaysTrue;

impl Tester for RcAlwaysTrue {
    fn test(&self) -> bool {
        true
    }
}

#[test]
fn test_rc_tester_observable_behavior() {
    let type_name = std::any::type_name::<RcTester>();
    assert!(type_name.contains("RcTester"), "{type_name}");
}

#[test]
fn test_rc_tester_not_operator_observable_behavior() {
    let owned_negated = !RcTester::new(|| true);
    assert!(!owned_negated.test());

    let original = RcTester::new(|| true);
    let borrowed_negated = !&original;
    assert!(!borrowed_negated.test());
    assert!(original.test());
}

/// Verifies naming, diagnostics, and clone-independent metadata updates.
#[test]
fn test_rc_tester_name_and_diagnostics() {
    let original = RcTester::new_with_name("ready", || true);
    let renamed = original.clone().with_name("renamed");

    assert_eq!(original.name(), Some("ready"));
    assert_eq!(renamed.name(), Some("renamed"));
    assert_eq!(format!("{original:?}"), "RcTester { name: Some(\"ready\") }");
    assert_eq!(format!("{original}"), "RcTester(ready)");
}

/// Verifies the direct trait method path and unnamed display diagnostics.
#[test]
fn test_rc_tester_trait_method_and_unnamed_display() {
    let tester = RcTester::new(|| true);

    assert!(Tester::test(&tester));
    assert_eq!(format!("{tester}"), "RcTester");
}

/// Verifies that Rc logical composition accepts semantic Tester objects.
#[test]
fn test_rc_tester_and_semantic_trait() {
    let first = RcTester::new(|| true);
    let combined = first.and(RcAlwaysTrue);

    assert!(combined.test());
    assert!(first.test());
}
