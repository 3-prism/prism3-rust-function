// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_function::BoxConditionalStatefulConsumer;

#[test]
fn test_box_conditional_stateful_consumer_observable_behavior() {
    let type_name = std::any::type_name::<BoxConditionalStatefulConsumer<i32>>();
    assert!(type_name.contains("BoxConditionalStatefulConsumer"), "{type_name}");
}
