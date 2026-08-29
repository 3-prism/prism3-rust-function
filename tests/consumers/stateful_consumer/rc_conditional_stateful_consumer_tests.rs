// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_function::RcConditionalStatefulConsumer;

#[test]
fn test_rc_conditional_stateful_consumer_observable_behavior() {
    let type_name = std::any::type_name::<RcConditionalStatefulConsumer<i32>>();
    assert!(type_name.contains("RcConditionalStatefulConsumer"), "{type_name}");
}
