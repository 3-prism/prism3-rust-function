// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Comprehensive tests for BiMutatingFunction trait and its implementations

use qubit_function::ArcBiMutatingFunction;
use qubit_function::ArcBiPredicate;
use qubit_function::BiMutatingFunction;
use qubit_function::BiMutatingFunctionOnce;
use qubit_function::BoxBiMutatingFunction;
use qubit_function::RcBiMutatingFunction;
use qubit_function::RcBiPredicate;

// ============================================================================
// Helper Functions and Data Structures
// ============================================================================

fn append_strings(x: &mut String, y: &mut String) -> usize {
    x.push_str("_modified");
    y.push_str("_changed");
    x.len() + y.len()
}

#[derive(Clone, Debug, PartialEq)]
struct TestStruct {
    value: i32,
}

impl TestStruct {
    fn new(value: i32) -> Self {
        Self { value }
    }

    fn modify(&mut self, other: &mut Self) -> i32 {
        self.value += other.value;
        other.value *= 2;
        self.value + other.value
    }
}

fn modify_structs(a: &mut TestStruct, b: &mut TestStruct) -> i32 {
    a.modify(b)
}

// ============================================================================
// BiMutatingFunction Trait Tests - Core Functionality
// ============================================================================

#[test]
fn test_rc_conditional_bi_mutating_function_clone() {
    let swap_and_sum = RcBiMutatingFunction::new(|x: &mut i32, y: &mut i32| {
        std::mem::swap(&mut *x, &mut *y);
        *x + *y
    });

    let multiply = RcBiMutatingFunction::new(|x: &mut i32, y: &mut i32| {
        *x *= *y;
        *x
    });

    let conditional = swap_and_sum.when(|x: &i32, _y: &i32| *x > 0).or_else(multiply);
    let cloned = conditional.clone();

    // Test original
    let mut a = 5;
    let mut b = 3;
    assert_eq!(conditional.apply(&mut a, &mut b), 8); // swap_and_sum executed

    // Test cloned (should behave identically)
    let mut c = 10;
    let mut d = 2;
    assert_eq!(cloned.apply(&mut c, &mut d), 12); // swap_and_sum executed
}

#[test]
fn test_arc_conditional_bi_mutating_function_clone() {
    let swap_and_sum = ArcBiMutatingFunction::new(|x: &mut i32, y: &mut i32| {
        std::mem::swap(&mut *x, &mut *y);
        *x + *y
    });

    let multiply = ArcBiMutatingFunction::new(|x: &mut i32, y: &mut i32| {
        *x *= *y;
        *x
    });

    let conditional = swap_and_sum.when(|x: &i32, _y: &i32| *x > 0).or_else(multiply);
    let cloned = conditional.clone();

    // Test original
    let mut a = 5;
    let mut b = 3;
    assert_eq!(conditional.apply(&mut a, &mut b), 8); // swap_and_sum executed

    // Test cloned (should behave identically)
    let mut c = 10;
    let mut d = 2;
    assert_eq!(cloned.apply(&mut c, &mut d), 12); // swap_and_sum executed
}

#[test]
fn test_impl_conditional_function_clone_three_params_bi_mutating_macro_coverage() {
    // Test to ensure the three-parameter version of
    // impl_conditional_function_clone macro is covered for bi-mutating
    // functions. This test verifies that the macro generates Clone
    // implementations for RcConditionalBiMutatingFunction<T, U, R> and
    // ArcConditionalBiMutatingFunction<T, U, R>

    // Test RcConditionalBiMutatingFunction (three parameters: T, U, R)
    {
        let swap = RcBiMutatingFunction::new(|x: &mut i32, y: &mut i32| {
            std::mem::swap(&mut *x, &mut *y);
            *x + *y
        });
        let pred = RcBiPredicate::new(|x: &i32, y: &i32| *x > *y);

        let conditional_rc = swap.when(pred);

        let cloned_rc = conditional_rc.clone();

        // Create or_else to test functionality
        let multiply = RcBiMutatingFunction::new(|x: &mut i32, y: &mut i32| {
            *x *= *y;
            *x
        });
        let func = cloned_rc.or_else(multiply);

        // Verify functionality
        let mut a1 = 5;
        let mut b1 = 3;
        assert_eq!(func.apply(&mut a1, &mut b1), 8); // when branch: 5 > 3, swapped: 3 + 5 = 8

        let mut a2 = 2;
        let mut b2 = 7;
        assert_eq!(func.apply(&mut a2, &mut b2), 14); // or_else branch: 2 <= 7, multiplied: 2 * 7 = 14
    }

    // Test ArcConditionalBiMutatingFunction (three parameters: T, U, R)
    {
        let increment = ArcBiMutatingFunction::new(|x: &mut i32, y: &mut i32| {
            *x += *y;
            *x
        });
        let pred = ArcBiPredicate::new(|x: &i32, _y: &i32| *x >= 0);

        let conditional_arc = increment.when(pred);

        let cloned_arc = conditional_arc.clone();

        // Create or_else to test functionality
        let decrement = ArcBiMutatingFunction::new(|x: &mut i32, y: &mut i32| {
            *x -= *y;
            *x
        });
        let func = cloned_arc.or_else(decrement);

        // Verify functionality
        let mut c1 = 5;
        let mut d1 = 3;
        assert_eq!(func.apply(&mut c1, &mut d1), 8); // when branch: increment: 5 + 3 = 8

        let mut c2 = -2;
        let mut d2 = 3;
        assert_eq!(func.apply(&mut c2, &mut d2), -5); // or_else branch: decrement: -2 - 3 = -5
    }
}

#[test]
fn test_conditional_bi_mutating_function_with_structs() {
    let modify = BoxBiMutatingFunction::new(modify_structs);
    let no_op = BoxBiMutatingFunction::new(|_a: &mut TestStruct, _b: &mut TestStruct| 0);

    let conditional = modify
        .when(|a: &TestStruct, b: &TestStruct| a.value > 0 && b.value > 0)
        .or_else(no_op);

    // Test when condition is true
    let mut s1 = TestStruct::new(10);
    let mut s2 = TestStruct::new(5);
    let result = conditional.apply(&mut s1, &mut s2);
    assert_eq!(result, 25); // modify executed: (10+5) + (5*2) = 15 + 10 = 25
    assert_eq!(s1.value, 15);
    assert_eq!(s2.value, 10);

    // Test when condition is false
    let mut s3 = TestStruct::new(-10);
    let mut s4 = TestStruct::new(5);
    let result2 = conditional.apply(&mut s3, &mut s4);
    assert_eq!(result2, 0); // no_op executed
    assert_eq!(s3.value, -10); // unchanged
    assert_eq!(s4.value, 5); // unchanged
}

// ============================================================================
// Edge Cases and Boundary Tests
// ============================================================================

#[test]
fn test_bi_mutating_function_with_zero_values() {
    let add = |x: &mut i32, y: &mut i32| {
        *x += *y;
        *x
    };

    let mut a = 0;
    let mut b = 0;
    assert_eq!(add.apply(&mut a, &mut b), 0);
    assert_eq!(a, 0);
    assert_eq!(b, 0);

    let mut c = 0;
    let mut d = 5;
    assert_eq!(add.apply(&mut c, &mut d), 5);
    assert_eq!(c, 5);
    assert_eq!(d, 5);
}

#[test]
fn test_bi_mutating_function_with_negative_values() {
    let multiply = |x: &mut i32, y: &mut i32| {
        *x *= *y;
        *x
    };

    let mut a = -5;
    let mut b = 3;
    assert_eq!(multiply.apply(&mut a, &mut b), -15);
    assert_eq!(a, -15);
    assert_eq!(b, 3);

    let mut c = -2;
    let mut d = -4;
    assert_eq!(multiply.apply(&mut c, &mut d), 8);
    assert_eq!(c, 8);
    assert_eq!(d, -4);
}

#[test]
fn test_bi_mutating_function_with_large_values() {
    let add = |x: &mut i64, y: &mut i64| {
        *x += *y;
        *x
    };

    let mut a = i64::MAX - 10;
    let mut b = 5;
    assert_eq!(add.apply(&mut a, &mut b), i64::MAX - 5);
    assert_eq!(a, i64::MAX - 5);
    assert_eq!(b, 5);
}

#[test]
fn test_bi_mutating_function_with_empty_strings() {
    let concat = |x: &mut String, y: &mut String| {
        x.push_str(y);
        x.len()
    };

    let mut s1 = String::new();
    let mut s2 = String::new();
    assert_eq!(concat.apply(&mut s1, &mut s2), 0);
    assert_eq!(s1, "");
    assert_eq!(s2, "");

    let mut s3 = "hello".to_string();
    let mut s4 = String::new();
    assert_eq!(concat.apply(&mut s3, &mut s4), 5);
    assert_eq!(s3, "hello");
    assert_eq!(s4, "");
}

#[test]
fn test_bi_mutating_function_with_unicode_strings() {
    let append = |x: &mut String, y: &mut String| {
        x.push('🌟');
        y.push('⭐');
        x.len() + y.len()
    };

    let mut s1 = "Hello".to_string();
    let mut s2 = "World".to_string();
    let result = append.apply(&mut s1, &mut s2);
    assert_eq!(s1, "Hello🌟");
    assert_eq!(s2, "World⭐");
    assert_eq!(result, 9 + 8); // "Hello🌟".len() + "World⭐".len()
}

#[test]
fn test_bi_mutating_function_identity_operations() {
    // Test functions that don't modify inputs
    let sum = |x: &mut i32, y: &mut i32| *x + *y;

    let mut a = 10;
    let mut b = 20;
    assert_eq!(sum.apply(&mut a, &mut b), 30);
    assert_eq!(a, 10); // unchanged
    assert_eq!(b, 20); // unchanged
}

#[test]
fn test_bi_mutating_function_chained_modifications() {
    let complex_op = |x: &mut i32, y: &mut i32| {
        *x = *x * 2 + *y;
        *y = *y * 3 - *x;
        *x + *y
    };

    let mut a = 3;
    let mut b = 5;
    let result = complex_op.apply(&mut a, &mut b);
    // a = 3*2 + 5 = 11
    // y = 5*3 - 11 = 15 - 11 = 4
    // result = 11 + 4 = 15
    assert_eq!(result, 15);
    assert_eq!(a, 11);
    assert_eq!(b, 4);
}

// ============================================================================
// Type Conversion Tests
// ============================================================================

// ============================================================================
// Error and Panic Tests
// ============================================================================

#[test]
#[should_panic]
fn test_bi_mutating_function_panic_in_closure() {
    let panic_func = |x: &mut i32, y: &mut i32| {
        if *x < 0 {
            panic!("Negative value not allowed");
        }
        *x + *y
    };

    let mut a = -5;
    let mut b = 10;
    let _ = panic_func.apply(&mut a, &mut b);
}

#[test]
fn test_bi_mutating_function_with_option_modification() {
    let modify_option = |x: &mut Option<i32>, y: &mut Option<i32>| {
        if let (Some(val1), Some(val2)) = (*x, *y) {
            *x = Some(val1 + val2);
            *y = Some(val1 * val2);
            val1 + val2
        } else {
            0
        }
    };

    let mut a = Some(10);
    let mut b = Some(5);
    let result = modify_option.apply(&mut a, &mut b);
    assert_eq!(result, 15);
    assert_eq!(a, Some(15));
    assert_eq!(b, Some(50));

    let mut c = None;
    let mut d = Some(5);
    let result2 = modify_option.apply(&mut c, &mut d);
    assert_eq!(result2, 0);
    assert_eq!(c, None);
    assert_eq!(d, Some(5));
}

// ============================================================================
// Integration with Other Function Types
// ============================================================================

// ============================================================================
// Custom BiMutatingFunction Implementation Tests - Test Trait Default Methods
// ============================================================================
