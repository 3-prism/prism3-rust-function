// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_function::BoxConditionalStatefulMutator;

#[test]
fn test_box_conditional_stateful_mutator_observable_behavior() {
    let type_name = std::any::type_name::<BoxConditionalStatefulMutator<i32>>();
    assert!(type_name.contains("BoxConditionalStatefulMutator"), "{type_name}");
}
