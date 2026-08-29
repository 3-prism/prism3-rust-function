// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_function::ArcStatefulBiPredicate;
use qubit_function::StatefulBiPredicate;

#[test]
fn test_arc_stateful_bi_predicate_observable_behavior() {
    let mut predicate = ArcStatefulBiPredicate::new(|left: &i32, right: &i32| left < right);
    assert!(predicate.test(&1, &2));
    assert!(!predicate.test(&2, &1));
}
