// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_function::BiPredicate;
use qubit_function::RcBiPredicate;

#[test]
fn test_rc_bi_predicate_observable_behavior() {
    let type_name = std::any::type_name::<RcBiPredicate<i32, i32>>();
    assert!(type_name.contains("RcBiPredicate"), "{type_name}");
}

#[test]
fn test_rc_bi_predicate_not_operator_observable_behavior() {
    let owned_negated = !RcBiPredicate::new(|first: &i32, second: &i32| first + second > 0);
    assert!(!owned_negated.test(&5, &3));
    assert!(owned_negated.test(&-5, &-3));

    let original = RcBiPredicate::new(|first: &i32, second: &i32| first + second > 0);
    let borrowed_negated = !&original;
    assert!(!borrowed_negated.test(&5, &3));
    assert!(borrowed_negated.test(&-5, &-3));
    assert!(original.test(&5, &3));
}
