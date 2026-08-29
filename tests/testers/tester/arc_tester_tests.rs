// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_function::testers::tester::ArcTester;
use qubit_function::testers::tester::Tester;

struct ArcAlwaysTrue;

impl Tester for ArcAlwaysTrue {
    fn test(&self) -> bool {
        true
    }
}

#[test]
fn test_arc_tester_observable_behavior() {
    let type_name = std::any::type_name::<ArcTester>();
    assert!(type_name.contains("ArcTester"), "{type_name}");
}

#[test]
fn test_arc_tester_not_operator_observable_behavior() {
    let owned_negated = !ArcTester::new(|| true);
    assert!(!owned_negated.test());

    let original = ArcTester::new(|| true);
    let borrowed_negated = !&original;
    assert!(!borrowed_negated.test());
    assert!(original.test());
}

/// Verifies naming, diagnostics, and clone-independent metadata updates.
#[test]
fn test_arc_tester_name_and_diagnostics() {
    let original = ArcTester::new_with_name("ready", || true);
    let renamed = original.clone().with_name("renamed");

    assert_eq!(original.name(), Some("ready"));
    assert_eq!(renamed.name(), Some("renamed"));
    assert_eq!(format!("{original:?}"), "ArcTester { name: Some(\"ready\") }");
    assert_eq!(format!("{original}"), "ArcTester(ready)");
}

/// Verifies that Arc logical composition accepts thread-safe Tester objects.
#[test]
fn test_arc_tester_and_semantic_trait() {
    let first = ArcTester::new(|| true);
    let combined = first.and(ArcAlwaysTrue);

    assert!(combined.test());
    assert!(first.test());
}
