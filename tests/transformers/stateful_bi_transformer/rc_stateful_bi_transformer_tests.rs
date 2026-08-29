// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_function::RcStatefulBiTransformer;

#[test]
fn test_rc_stateful_bi_transformer_observable_behavior() {
    let type_name = std::any::type_name::<RcStatefulBiTransformer<i32, i32, i32>>();
    assert!(type_name.contains("RcStatefulBiTransformer"), "{type_name}");
}
