// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_function::RcCallableWith;

#[test]
fn test_rc_callable_with_observable_behavior() {
    let type_name = std::any::type_name::<RcCallableWith<i32, i32, std::io::Error>>();
    assert!(type_name.contains("RcCallableWith"), "{type_name}");
}
