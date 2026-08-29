// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Comprehensive tests for Function trait and its implementations

use qubit_function::ArcFunction;
use qubit_function::ArcPredicate;
use qubit_function::BoxFunction;
use qubit_function::BoxPredicate;
use qubit_function::Function;
use qubit_function::FunctionOnce;
use qubit_function::Predicate;
use qubit_function::RcFunction;
use qubit_function::RcPredicate;

// ============================================================================
// Function Trait Tests - Core Functionality
// ============================================================================

#[test]
fn test_function_trait_apply() {
    // Test that Function trait's apply method works correctly
    let double = |x: &i32| x * 2;
    assert_eq!(double.apply(&21), 42);
    assert_eq!(double.apply(&0), 0);
    assert_eq!(double.apply(&-10), -20);
}

#[test]
fn test_box_function_new_allows_non_static_t() {
    fn run<'a>(value: &'a str) -> usize {
        let func: BoxFunction<&'a str, usize> = BoxFunction::new(|x: &&'a str| x.len());
        func.apply(&value)
    }

    let text = String::from("hello");
    assert_eq!(run(text.as_str()), 5);
}

#[test]
fn test_box_function_new_allows_non_static_r() {
    fn run<'a>(value: &'a str) -> &'a str {
        let func: BoxFunction<&'a str, &'a str> = BoxFunction::new(|x: &&'a str| *x);
        func.apply(&value)
    }

    let text = String::from("qubit");
    assert_eq!(run(text.as_str()), "qubit");
}

#[test]
fn test_rc_function_new_allows_non_static_t() {
    fn run<'a>(value: &'a str) -> usize {
        let func: RcFunction<&'a str, usize> = RcFunction::new(|x: &&'a str| x.len());
        func.apply(&value)
    }

    let text = String::from("hello");
    assert_eq!(run(text.as_str()), 5);
}

#[test]
fn test_arc_function_new_allows_non_static_t() {
    fn run<'a>(value: &'a str) -> usize {
        let func: ArcFunction<&'a str, usize> = ArcFunction::new(|x: &&'a str| x.len() + 1);
        func.apply(&value)
    }

    let text = String::from("hello");
    assert_eq!(run(text.as_str()), 6);
}

#[test]
fn test_rc_function_new_allows_non_static_r() {
    fn run<'a>(value: &'a str) -> &'a str {
        let func: RcFunction<&'a str, &'a str> = RcFunction::new(|x: &&'a str| *x);
        func.apply(&value)
    }

    let text = String::from("qubit");
    assert_eq!(run(text.as_str()), "qubit");
}

// ============================================================================
// BoxFunction Tests - Constructor and Basic Operations
// ============================================================================

#[test]
fn test_box_function_new() {
    // Test BoxFunction::new with simple closure
    let double = BoxFunction::new(|x: &i32| x * 2);
    assert_eq!(double.apply(&21), 42);
    assert_eq!(double.apply(&0), 0);
    assert_eq!(double.apply(&-5), -10);
}

#[test]
fn test_box_function_identity() {
    // Test BoxFunction::identity
    let identity = BoxFunction::<i32, i32>::identity();
    assert_eq!(identity.apply(&42), 42);
    assert_eq!(identity.apply(&0), 0);
    assert_eq!(identity.apply(&-100), -100);
}

#[test]
fn test_box_function_constant() {
    // Test BoxFunction::constant
    let constant = BoxFunction::constant("hello");
    assert_eq!(constant.apply(&123), "hello");
    assert_eq!(constant.apply(&456), "hello");
    assert_eq!(constant.apply(&0), "hello");
}

#[test]
fn test_box_function_apply() {
    // Test Function trait implementation for BoxFunction
    let add_one = BoxFunction::new(|x: &i32| x + 1);
    assert_eq!(add_one.apply(&41), 42);
    assert_eq!(add_one.apply(&0), 1);
    assert_eq!(add_one.apply(&-1), 0);
}

// ============================================================================
// BoxFunction Tests - Composition Methods
// ============================================================================

#[test]
fn test_box_function_and_then() {
    // Test BoxFunction::and_then composition
    let double = BoxFunction::new(|x: &i32| x * 2);
    let to_string = BoxFunction::new(|x: &i32| x.to_string());
    let composed = double.and_then(to_string);
    assert_eq!(composed.apply(&21), "42");
    assert_eq!(composed.apply(&0), "0");
    assert_eq!(composed.apply(&-5), "-10");
}

#[test]
fn test_box_function_and_then_with_closure() {
    // Test and_then with closure
    let double = BoxFunction::new(|x: &i32| x * 2);
    let composed = double.and_then(|x: &i32| x + 10);
    assert_eq!(composed.apply(&16), 42);
}

// ============================================================================
// BoxFunction Tests - Conditional Execution
// ============================================================================

#[test]
fn test_box_function_when_or_else() {
    // Test conditional execution with when/or_else
    let double = BoxFunction::new(|x: &i32| x * 2);
    let identity = BoxFunction::<i32, i32>::identity();
    let conditional = double.when(|x: &i32| *x > 0).or_else(identity);

    assert_eq!(conditional.apply(&5), 10); // when branch
    assert_eq!(conditional.apply(&-5), -5); // or_else branch
    assert_eq!(conditional.apply(&0), 0); // or_else branch
}

#[test]
fn test_box_function_when_with_closure() {
    // Test when with closure predicate and or_else
    let double = BoxFunction::new(|x: &i32| x * 2);
    let conditional = double.when(|x: &i32| *x >= 10).or_else(|x: &i32| *x);

    assert_eq!(conditional.apply(&15), 30); // when branch
    assert_eq!(conditional.apply(&5), 5); // or_else branch
}

#[test]
fn test_box_function_when_with_predicate() {
    // Test when with BoxPredicate
    let double = BoxFunction::new(|x: &i32| x * 2);
    let is_positive = BoxPredicate::new(|x: &i32| *x > 0);
    let conditional = double.when(is_positive).or_else(|x: &i32| -(*x));

    assert_eq!(conditional.apply(&5), 10); // when branch
    assert_eq!(conditional.apply(&-5), 5); // or_else branch
}

// ============================================================================
// BoxFunction Tests - Type Conversions (Function trait)
// ============================================================================

// ============================================================================
// BoxFunction Tests - FunctionOnce Implementation
// ============================================================================

#[test]
fn test_box_function_once_impl_apply() {
    // Test FunctionOnce::apply for BoxFunction
    let double = BoxFunction::new(|x: &i32| x * 2);
    let result = double.apply(&21);
    assert_eq!(result, 42);
}

// ============================================================================
// ArcFunction Tests - Constructor and Basic Operations
// ============================================================================

#[test]
fn test_arc_function_new() {
    // Test ArcFunction::new with simple closure
    let double = ArcFunction::new(|x: &i32| x * 2);
    assert_eq!(double.apply(&21), 42);
    assert_eq!(double.apply(&0), 0);
    assert_eq!(double.apply(&-5), -10);
}

#[test]
fn test_arc_function_identity() {
    // Test ArcFunction::identity
    let identity = ArcFunction::<i32, i32>::identity();
    assert_eq!(identity.apply(&42), 42);
    assert_eq!(identity.apply(&0), 0);
    assert_eq!(identity.apply(&-100), -100);
}

#[test]
fn test_arc_function_constant() {
    // Test ArcFunction::constant
    let constant = ArcFunction::constant("hello");
    assert_eq!(constant.apply(&123), "hello");
    assert_eq!(constant.apply(&456), "hello");
}

#[test]
fn test_arc_function_once_impl_apply() {
    // Test Function trait implementation for ArcFunction
    let add_one = ArcFunction::new(|x: &i32| x + 1);
    assert_eq!(add_one.apply(&41), 42);
    assert_eq!(add_one.apply(&0), 1);
}

#[test]
fn test_arc_function_clone() {
    // Test ArcFunction::clone
    let double = ArcFunction::new(|x: &i32| x * 2);
    let cloned = double.clone();
    assert_eq!(double.apply(&21), 42);
    assert_eq!(cloned.apply(&21), 42);
}

// ============================================================================
// ArcFunction Tests - Composition Methods
// ============================================================================

#[test]
fn test_arc_function_and_then() {
    // Test ArcFunction::and_then composition
    let double = ArcFunction::new(|x: &i32| x * 2);
    let to_string = ArcFunction::new(|x: &i32| x.to_string());
    let composed = double.and_then(to_string);
    assert_eq!(composed.apply(&21), "42");
    // Original still usable
    assert_eq!(double.apply(&10), 20);
}

#[test]
fn test_arc_function_and_then_with_clone() {
    // Test and_then preserving original with clone
    let double = ArcFunction::new(|x: &i32| x * 2);
    let to_string = ArcFunction::new(|x: &i32| x.to_string());
    let composed = double.and_then(to_string.clone());
    assert_eq!(composed.apply(&21), "42");
    assert_eq!(to_string.apply(&5), "5");
}

// ============================================================================
// ArcFunction Tests - Conditional Execution
// ============================================================================

#[test]
fn test_arc_function_when_or_else() {
    // Test conditional execution with when/or_else
    let double = ArcFunction::new(|x: &i32| x * 2);
    let identity = ArcFunction::<i32, i32>::identity();
    let conditional = double.when(|x: &i32| *x > 0).or_else(identity);

    let conditional_clone = conditional.clone();
    assert_eq!(conditional.apply(&5), 10);
    assert_eq!(conditional_clone.apply(&-5), -5);
}

#[test]
fn test_arc_function_when_with_predicate() {
    // Test when with ArcPredicate
    let double = ArcFunction::new(|x: &i32| x * 2);
    let is_positive = ArcPredicate::new(|x: &i32| *x > 0);
    let conditional = double.when(is_positive.clone()).or_else(ArcFunction::identity());

    assert_eq!(conditional.apply(&5), 10);
    assert!(is_positive.test(&3));
}

// ============================================================================
// ArcFunction Tests - Type Conversions
// ============================================================================

// ============================================================================
// ArcFunction Tests - FunctionOnce Implementation
// ============================================================================

// ============================================================================
// ArcFunction Tests - Thread Safety
// ============================================================================

#[test]
fn test_arc_function_send_sync() {
    // Test that ArcFunction is Send + Sync
    let double = ArcFunction::new(|x: &i32| x * 2);
    let double_clone = double.clone();

    let handle = std::thread::spawn(move || double_clone.apply(&21));

    assert_eq!(handle.join().expect("thread should not panic"), 42);
    assert_eq!(double.apply(&10), 20);
}

// ============================================================================
// RcFunction Tests - Constructor and Basic Operations
// ============================================================================

#[test]
fn test_rc_function_new() {
    // Test RcFunction::new with simple closure
    let double = RcFunction::new(|x: &i32| x * 2);
    assert_eq!(double.apply(&21), 42);
    assert_eq!(double.apply(&0), 0);
    assert_eq!(double.apply(&-5), -10);
}

#[test]
fn test_rc_function_identity() {
    // Test RcFunction::identity
    let identity = RcFunction::<i32, i32>::identity();
    assert_eq!(identity.apply(&42), 42);
    assert_eq!(identity.apply(&0), 0);
    assert_eq!(identity.apply(&-100), -100);
}

#[test]
fn test_rc_function_constant() {
    // Test RcFunction::constant
    let constant = RcFunction::constant("hello");
    assert_eq!(constant.apply(&123), "hello");
    assert_eq!(constant.apply(&456), "hello");
}

#[test]
fn test_rc_function_once_impl_apply() {
    // Test Function trait implementation for RcFunction
    let add_one = RcFunction::new(|x: &i32| x + 1);
    assert_eq!(add_one.apply(&41), 42);
    assert_eq!(add_one.apply(&0), 1);
}

#[test]
fn test_rc_function_clone() {
    // Test RcFunction::clone
    let double = RcFunction::new(|x: &i32| x * 2);
    let cloned = double.clone();
    assert_eq!(double.apply(&21), 42);
    assert_eq!(cloned.apply(&21), 42);
}

// ============================================================================
// RcFunction Tests - Composition Methods
// ============================================================================

#[test]
fn test_rc_function_and_then() {
    // Test RcFunction::and_then composition
    let double = RcFunction::new(|x: &i32| x * 2);
    let to_string = RcFunction::new(|x: &i32| x.to_string());
    let composed = double.and_then(to_string);
    assert_eq!(composed.apply(&21), "42");
    // Original still usable
    assert_eq!(double.apply(&10), 20);
}

#[test]
fn test_rc_function_and_then_with_clone() {
    // Test and_then preserving original with clone
    let double = RcFunction::new(|x: &i32| x * 2);
    let to_string = RcFunction::new(|x: &i32| x.to_string());
    let composed = double.and_then(to_string.clone());
    assert_eq!(composed.apply(&21), "42");
    assert_eq!(to_string.apply(&5), "5");
}

// ============================================================================
// RcFunction Tests - Conditional Execution
// ============================================================================

#[test]
fn test_rc_function_when_or_else() {
    // Test conditional execution with when/or_else
    let double = RcFunction::new(|x: &i32| x * 2);
    let identity = RcFunction::<i32, i32>::identity();
    let conditional = double.when(|x: &i32| *x > 0).or_else(identity);

    let conditional_clone = conditional.clone();
    assert_eq!(conditional.apply(&5), 10);
    assert_eq!(conditional_clone.apply(&-5), -5);
}

#[test]
fn test_rc_function_when_with_predicate() {
    // Test when with RcPredicate
    let double = RcFunction::new(|x: &i32| x * 2);
    let is_positive = RcPredicate::new(|x: &i32| *x > 0);
    let conditional = double
        .when(is_positive.clone())
        .or_else(RcFunction::<i32, i32>::identity());

    assert_eq!(conditional.apply(&5), 10);
    assert!(is_positive.test(&3));
}

// ============================================================================
// RcFunction Tests - Type Conversions
// ============================================================================

// ============================================================================
// RcFunction Tests - FunctionOnce Implementation
// ============================================================================

// ============================================================================
// Edge Cases and Boundary Tests
// ============================================================================

#[test]
fn test_function_with_zero() {
    // Test functions with zero input
    let double = BoxFunction::new(|x: &i32| x * 2);
    assert_eq!(double.apply(&0), 0);
}

#[test]
fn test_function_with_negative() {
    // Test functions with negative input
    let double = BoxFunction::new(|x: &i32| x * 2);
    assert_eq!(double.apply(&-42), -84);
}

#[test]
fn test_function_with_max_value() {
    // Test functions with maximum value
    let identity = BoxFunction::<i32, i32>::identity();
    assert_eq!(identity.apply(&i32::MAX), i32::MAX);
}

#[test]
fn test_function_with_min_value() {
    // Test functions with minimum value
    let identity = BoxFunction::<i32, i32>::identity();
    assert_eq!(identity.apply(&i32::MIN), i32::MIN);
}
