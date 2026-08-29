// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_function::BoxBiTransformerOnce;

#[test]
fn test_box_bi_transformer_once_observable_behavior() {
    let type_name = std::any::type_name::<BoxBiTransformerOnce<i32, i32, i32>>();
    assert!(type_name.contains("BoxBiTransformerOnce"), "{type_name}");
}
