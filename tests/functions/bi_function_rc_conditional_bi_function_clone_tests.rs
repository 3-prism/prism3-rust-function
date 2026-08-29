// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Comprehensive tests for BiFunction trait and its implementations

use qubit_function::ArcBiFunction;
use qubit_function::ArcBiPredicate;
use qubit_function::BiFunction;
use qubit_function::BiFunctionOnce;
use qubit_function::BoxBiFunction;
use qubit_function::RcBiFunction;
use qubit_function::RcBiPredicate;

// ============================================================================
// BiFunction Trait Tests - Core Functionality
// ============================================================================

#[test]
fn test_rc_conditional_bi_function_clone() {
    let add = RcBiFunction::new(|x: &i32, y: &i32| *x + *y);
    let multiply = RcBiFunction::new(|x: &i32, y: &i32| *x * *y);

    let conditional = add.when(|x: &i32, y: &i32| *x > 0 && *y > 0).or_else(multiply);
    let cloned = conditional.clone();

    // Test original
    assert_eq!(conditional.apply(&3, &4), 7); // when branch: 3 + 4 = 7
    assert_eq!(conditional.apply(&-3, &4), -12); // or_else branch: -3 * 4 = -12

    // Test cloned (should behave identically)
    assert_eq!(cloned.apply(&3, &4), 7); // when branch: 3 + 4 = 7
    assert_eq!(cloned.apply(&-3, &4), -12); // or_else branch: -3 * 4 = -12
}

#[test]
fn test_arc_conditional_bi_function_clone() {
    let add = ArcBiFunction::new(|x: &i32, y: &i32| *x + *y);
    let multiply = ArcBiFunction::new(|x: &i32, y: &i32| *x * *y);

    let conditional = add.when(|x: &i32, y: &i32| *x > 0 && *y > 0).or_else(multiply);
    let cloned = conditional.clone();

    // Test original
    assert_eq!(conditional.apply(&3, &4), 7); // when branch: 3 + 4 = 7
    assert_eq!(conditional.apply(&-3, &4), -12); // or_else branch: -3 * 4 = -12

    // Test cloned (should behave identically)
    assert_eq!(cloned.apply(&3, &4), 7); // when branch: 3 + 4 = 7
    assert_eq!(cloned.apply(&-3, &4), -12); // or_else branch: -3 * 4 = -12
}

#[test]
fn test_impl_conditional_function_clone_three_params_macro_coverage() {
    // Test to ensure the three-parameter version of
    // impl_conditional_function_clone macro is covered This test verifies
    // that the macro generates Clone implementations for three-parameter
    // structs by testing that RcConditionalBiFunction and
    // ArcConditionalBiFunction implement Clone

    // Test RcConditionalBiFunction (three parameters: T, U, R)
    {
        let add = RcBiFunction::new(|x: &i32, y: &i32| *x + *y);
        let pred = RcBiPredicate::new(|x: &i32, y: &i32| *x > 0 && *y > 0);

        let conditional_rc = add.when(pred);

        let cloned_rc = conditional_rc.clone();

        // Create or_else to test functionality
        let multiply = RcBiFunction::new(|x: &i32, y: &i32| *x * *y);
        let func = cloned_rc.or_else(multiply);

        // Verify functionality
        assert_eq!(func.apply(&3, &4), 7); // when branch
        assert_eq!(func.apply(&-3, &4), -12); // or_else branch: -3 * 4 = -12
    }

    // Test ArcConditionalBiFunction (three parameters: T, U, R)
    {
        let subtract = ArcBiFunction::new(|x: &i32, y: &i32| *x - *y);
        let pred = ArcBiPredicate::new(|x: &i32, y: &i32| *x >= *y);

        let conditional_arc = subtract.when(pred);

        let cloned_arc = conditional_arc.clone();

        // Create or_else to test functionality
        let negate = ArcBiFunction::new(|x: &i32, y: &i32| -*x - *y);
        let func = cloned_arc.or_else(negate);

        // Verify functionality
        assert_eq!(func.apply(&5, &3), 2); // when branch: 5 - 3 = 2
        assert_eq!(func.apply(&3, &5), -8); // or_else branch: -(3 + 5) = -8
    }
}

// ============================================================================
// Advanced Composition Tests
// ============================================================================

// ============================================================================
// Thread Safety Tests for ArcBiFunction
// ============================================================================

#[test]
fn test_arc_bi_function_and_then() {
    use qubit_function::ArcFunction;

    let add = ArcBiFunction::new(|x: &i32, y: &i32| *x + *y);
    let multiply_by_two = ArcFunction::new(|x: &i32| *x * 2);

    let chained = add.and_then(multiply_by_two);
    assert_eq!(chained.apply(&2, &3), 10); // (2+3) * 2 = 10
}

#[test]
fn test_arc_bi_function_thread_safety() {
    use std::thread;

    let func = ArcBiFunction::new(|x: &i32, y: &i32| *x + *y);
    let func_clone = func.clone();

    let handle = thread::spawn(move || func_clone.apply(&10, &20));

    let result = handle.join().expect("thread should not panic");
    assert_eq!(result, 30);
    assert_eq!(func.apply(&10, &20), 30);
}
