// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_function::BoxConditionalBiFunctionOnce;

#[test]
fn test_box_conditional_bi_function_once_observable_behavior() {
    let type_name = std::any::type_name::<BoxConditionalBiFunctionOnce<i32, i32, i32>>();
    assert!(type_name.contains("BoxConditionalBiFunctionOnce"), "{type_name}");
}
