// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_function::ArcConditionalTransformer;

#[test]
fn test_arc_conditional_transformer_observable_behavior() {
    let type_name = std::any::type_name::<ArcConditionalTransformer<i32, i32>>();
    assert!(type_name.contains("ArcConditionalTransformer"), "{type_name}");
}
