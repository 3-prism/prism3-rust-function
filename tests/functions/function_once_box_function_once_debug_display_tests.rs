// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Comprehensive tests for FunctionOnce trait and BoxFunctionOnce

use qubit_function::BoxFunctionOnce;
use qubit_function::FunctionOnce;
use qubit_function::Predicate;
use qubit_function::RcPredicate;

// ============================================================================
// FunctionOnce Trait Tests - Core Functionality
// ============================================================================

#[test]
fn test_box_function_once_debug_display() {
    // Test Debug and Display for BoxFunctionOnce without name
    let double = BoxFunctionOnce::new(|x: &i32| x * 2);
    let debug_str = format!("{:?}", double);
    assert!(debug_str.contains("BoxFunctionOnce"));
    assert!(debug_str.contains("name"));
    assert!(debug_str.contains("function"));

    let display_str = format!("{}", double);
    assert_eq!(display_str, "BoxFunctionOnce");

    // Test Debug and Display for BoxFunctionOnce with name
    let named_double = BoxFunctionOnce::new_with_name("double", |x: &i32| x * 2);
    let named_debug_str = format!("{:?}", named_double);
    assert!(named_debug_str.contains("BoxFunctionOnce"));
    assert!(named_debug_str.contains("name"));
    assert!(named_debug_str.contains("function"));

    let named_display_str = format!("{}", named_double);
    assert_eq!(named_display_str, "BoxFunctionOnce(double)");
}

// ============================================================================
// FunctionOnce Name Management Tests
// ============================================================================

#[test]
fn test_box_function_once_name_methods() {
    // Test new_with_name, name(), and set_name()
    let mut double = BoxFunctionOnce::new_with_name("double_func_once", |x: &i32| x * 2);

    // Test name() returns the initial name
    assert_eq!(double.name(), Some("double_func_once"));

    // Test set_name() changes the name
    double.set_name("modified_double_once");
    assert_eq!(double.name(), Some("modified_double_once"));

    // Test that function still works after name change
    assert_eq!(double.apply(&5), 10);
}

// ============================================================================
// ConditionalFunctionOnce Debug and Display Tests
// ============================================================================

#[test]
fn test_box_conditional_function_once_debug_display() {
    // Test Debug and Display for BoxConditionalFunctionOnce without name
    let double = BoxFunctionOnce::new(|x: &i32| x * 2);
    let conditional = double.when(|x: &i32| *x > 0);

    let debug_str = format!("{:?}", conditional);
    assert!(debug_str.contains("BoxConditionalFunctionOnce"));
    assert!(debug_str.contains("name"));
    assert!(debug_str.contains("function"));
    assert!(debug_str.contains("predicate"));

    let display_str = format!("{}", conditional);
    assert!(display_str.starts_with("BoxConditionalFunctionOnce("));
    assert!(display_str.contains("BoxFunctionOnce"));
    assert!(display_str.contains("BoxPredicate"));
    assert!(display_str.ends_with(")"));

    // Test Debug and Display for BoxConditionalFunctionOnce with name
    let triple = BoxFunctionOnce::new_with_name("triple_func_once", |x: &i32| x * 3);
    let named_conditional = triple.when(|x: &i32| *x % 2 == 0);

    let named_debug_str = format!("{:?}", named_conditional);
    assert!(named_debug_str.contains("BoxConditionalFunctionOnce"));
    assert!(named_debug_str.contains("name"));
    assert!(named_debug_str.contains("function"));
    assert!(named_debug_str.contains("predicate"));

    let named_display_str = format!("{}", named_conditional);
    assert!(named_display_str.starts_with("BoxConditionalFunctionOnce("));
    assert!(named_display_str.contains("BoxFunctionOnce(triple_func_once)"));
    assert!(named_display_str.contains("BoxPredicate"));
    assert!(named_display_str.ends_with(")"));
}
