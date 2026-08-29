// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_function::testers::tester::BoxTester;
use qubit_function::testers::tester::Tester;

struct BoxAlwaysTrue;

impl Tester for BoxAlwaysTrue {
    fn test(&self) -> bool {
        true
    }
}

#[test]
fn test_box_tester_observable_behavior() {
    let type_name = std::any::type_name::<BoxTester>();
    assert!(type_name.contains("BoxTester"), "{type_name}");
}

/// Verifies naming and diagnostic formatting for `BoxTester`.
#[test]
fn test_box_tester_name_and_diagnostics() {
    let mut tester = BoxTester::new_with_optional_name(|| true, Some("ready".to_owned()));

    assert_eq!(tester.name(), Some("ready"));
    assert_eq!(format!("{tester:?}"), "BoxTester { name: Some(\"ready\") }");
    assert_eq!(format!("{tester}"), "BoxTester(ready)");

    tester.clear_name();
    assert_eq!(tester.name(), None);
    tester.set_name("healthy");
    assert_eq!(tester.name(), Some("healthy"));
}

/// Verifies owned logical combinators and direct trait invocation.
#[test]
fn test_box_tester_logical_combinators_and_trait_paths() {
    assert!(Tester::test(&BoxTester::new(|| true)));
    assert!(!BoxTester::new(|| true).and(|| false).test());
    assert!(BoxTester::new(|| false).or(|| true).test());
    assert!(!BoxTester::new(|| true).nand(BoxAlwaysTrue).test());
    assert!(BoxTester::new(|| true).xor(|| false).test());
    assert!(BoxTester::new(|| false).nor(|| false).test());
}

/// Verifies owned negation and unnamed display diagnostics.
#[test]
fn test_box_tester_not_operator_and_unnamed_display() {
    let negated = !BoxTester::new(|| true);

    assert!(!negated.test());
    assert_eq!(format!("{}", BoxTester::new(|| true)), "BoxTester");
}
