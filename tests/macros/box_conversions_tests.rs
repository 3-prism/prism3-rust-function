// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_function::BoxFunction;
use qubit_function::Function;

#[test]
fn test_box_conversions_observable_behavior() {
    let mut function = BoxFunction::new_with_name("smoke", |value: &i32| value + 1);
    assert_eq!(function.name(), Some("smoke"));
    function.set_name("renamed");
    assert_eq!(function.apply(&41), 42);
    assert_eq!(format!("{}", function), "BoxFunction(renamed)");
}
