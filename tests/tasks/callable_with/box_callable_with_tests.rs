// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_function::BoxCallableWith;

#[test]
fn test_box_callable_with_observable_behavior() {
    let type_name = std::any::type_name::<BoxCallableWith<i32, i32, std::io::Error>>();
    assert!(type_name.contains("BoxCallableWith"), "{type_name}");
}
