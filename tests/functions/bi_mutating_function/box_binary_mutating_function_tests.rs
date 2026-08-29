// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_function::BiMutatingFunction;
use qubit_function::BoxBinaryMutatingFunction;

#[test]
fn test_box_binary_mutating_function_alias() {
    let function = BoxBinaryMutatingFunction::new(|a: &mut i32, b: &mut i32| {
        *a += 1;
        *b += 1;
        *a + *b
    });
    let (mut a, mut b) = (20, 20);
    assert_eq!(function.apply(&mut a, &mut b), 42);
}
