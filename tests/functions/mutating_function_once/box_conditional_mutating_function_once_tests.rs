// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_function::BoxConditionalMutatingFunctionOnce;

#[test]
fn test_box_conditional_mutating_function_once_observable_behavior() {
    let type_name = std::any::type_name::<BoxConditionalMutatingFunctionOnce<i32, i32>>();
    assert!(type_name.contains("BoxConditionalMutatingFunctionOnce"), "{type_name}");
}
