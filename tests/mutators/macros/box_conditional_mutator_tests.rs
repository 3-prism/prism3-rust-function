// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_function::BoxMutator;
use qubit_function::Mutator;

#[test]
fn test_box_conditional_mutator_macro_behavior() {
    let mutator = BoxMutator::new(|value: &mut i32| *value += 1).when(|value: &i32| *value > 0);
    let mut positive = 41;
    mutator.apply(&mut positive);
    let mut negative = -1;
    mutator.apply(&mut negative);
    assert_eq!(positive, 42);
    assert_eq!(negative, -1);
}
