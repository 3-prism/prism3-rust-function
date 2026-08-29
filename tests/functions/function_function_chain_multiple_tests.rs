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
fn test_function_chain_multiple() {
    // Test chaining multiple functions
    let add_one = BoxFunction::new(|x: &i32| x + 1);
    let double = BoxFunction::new(|x: &i32| x * 2);
    let add_ten = BoxFunction::new(|x: &i32| x + 10);

    let composed = add_one.and_then(double).and_then(add_ten);
    assert_eq!(composed.apply(&5), 22); // ((5 + 1) * 2) + 10
}

#[test]
fn test_function_with_string() {
    // Test functions with String type
    let to_upper = BoxFunction::new(|s: &String| s.to_uppercase());
    let input = String::from("hello");
    assert_eq!(to_upper.apply(&input), "HELLO");
}

#[test]
fn test_function_with_vec() {
    // Test functions with Vec type
    let get_len = BoxFunction::new(|v: &Vec<i32>| v.len());
    let vec = vec![1, 2, 3, 4, 5];
    assert_eq!(get_len.apply(&vec), 5);
}

#[test]
fn test_function_with_option() {
    // Test functions with Option type
    let unwrap_or_zero = BoxFunction::new(|opt: &Option<i32>| opt.unwrap_or(0));
    assert_eq!(unwrap_or_zero.apply(&Some(42)), 42);
    assert_eq!(unwrap_or_zero.apply(&None), 0);
}

#[test]
fn test_conditional_function_edge_cases() {
    // Test conditional function with boundary values
    let double = BoxFunction::new(|x: &i32| x * 2);
    let negate = BoxFunction::new(|x: &i32| -(*x));
    let conditional = double.when(|x: &i32| *x >= 0).or_else(negate);

    assert_eq!(conditional.apply(&0), 0); // Boundary: zero
    assert_eq!(conditional.apply(&1), 2); // Positive
    assert_eq!(conditional.apply(&-1), 1); // Negative
}

// ============================================================================
// Concrete wrapper composition tests
// ============================================================================

// ============================================================================
// ArcConditionalFunction Clone Tests
// ============================================================================

#[test]
fn test_arc_conditional_function_clone() {
    let double = ArcFunction::new(|x: &i32| x * 2);
    let conditional = double.when(|x: &i32| *x > 0);

    // Clone conditional function
    let conditional_clone = conditional.clone();

    // Both cloned conditional functions work properly
    let result1 = conditional.or_else(|x: &i32| -(*x));
    let result2 = conditional_clone.or_else(|x: &i32| x + 100);

    assert_eq!(result1.apply(&5), 10); // Condition met: 5 * 2
    assert_eq!(result1.apply(&-5), 5); // Condition not met: -(-5)
    assert_eq!(result2.apply(&5), 10); // Condition met: 5 * 2
    assert_eq!(result2.apply(&-5), 95); // Condition not met: -5 + 100
}

#[test]
fn test_arc_conditional_function_clone_multiple() {
    let triple = ArcFunction::new(|x: &i32| x * 3);
    let conditional = triple.when(|x: &i32| *x % 2 == 0);

    // Create multiple clones
    let clone1 = conditional.clone();
    let clone2 = conditional.clone();
    let clone3 = conditional.clone();

    let result1 = conditional.or_else(|x: &i32| *x);
    let result2 = clone1.or_else(|x: &i32| *x);
    let result3 = clone2.or_else(|x: &i32| *x);
    let result4 = clone3.or_else(|x: &i32| *x);

    // All clones work properly
    assert_eq!(result1.apply(&4), 12); // Even number: 4 * 3
    assert_eq!(result2.apply(&4), 12);
    assert_eq!(result3.apply(&4), 12);
    assert_eq!(result4.apply(&4), 12);

    assert_eq!(result1.apply(&5), 5); // Odd number: 5
    assert_eq!(result2.apply(&5), 5);
    assert_eq!(result3.apply(&5), 5);
    assert_eq!(result4.apply(&5), 5);
}

// ============================================================================
// ConditionalFunction Debug and Display Tests
// ============================================================================

#[test]
fn test_box_conditional_function_debug_display() {
    // Test Debug and Display for BoxConditionalFunction without name
    let double = BoxFunction::new(|x: &i32| x * 2);
    let conditional = double.when(|x: &i32| *x > 0);

    let debug_str = format!("{:?}", conditional);
    assert!(debug_str.contains("BoxConditionalFunction"));
    assert!(debug_str.contains("name"));
    assert!(debug_str.contains("function"));
    assert!(debug_str.contains("predicate"));

    let display_str = format!("{}", conditional);
    assert!(display_str.starts_with("BoxConditionalFunction("));
    assert!(display_str.contains("BoxFunction"));
    assert!(display_str.contains("BoxPredicate"));
    assert!(display_str.ends_with(")"));

    // Test Debug and Display for BoxConditionalFunction with name
    let triple = BoxFunction::new_with_name("triple_func", |x: &i32| x * 3);
    let named_conditional = triple.when(|x: &i32| *x % 2 == 0);

    let named_debug_str = format!("{:?}", named_conditional);
    assert!(named_debug_str.contains("BoxConditionalFunction"));
    assert!(named_debug_str.contains("name"));
    assert!(named_debug_str.contains("function"));
    assert!(named_debug_str.contains("predicate"));

    let named_display_str = format!("{}", named_conditional);
    assert!(named_display_str.starts_with("BoxConditionalFunction("));
    assert!(named_display_str.contains("BoxFunction(triple_func)"));
    assert!(named_display_str.contains("BoxPredicate"));
    assert!(named_display_str.ends_with(")"));
}

#[test]
fn test_rc_conditional_function_debug_display() {
    // Test Debug and Display for RcConditionalFunction without name
    let double = RcFunction::new(|x: &i32| x * 2);
    let conditional = double.when(|x: &i32| *x > 0);

    let debug_str = format!("{:?}", conditional);
    assert!(debug_str.contains("RcConditionalFunction"));
    assert!(debug_str.contains("name"));
    assert!(debug_str.contains("function"));
    assert!(debug_str.contains("predicate"));

    let display_str = format!("{}", conditional);
    assert!(display_str.starts_with("RcConditionalFunction("));
    assert!(display_str.contains("RcFunction"));
    assert!(display_str.contains("RcPredicate"));
    assert!(display_str.ends_with(")"));

    // Test Debug and Display for RcConditionalFunction with name
    let triple = RcFunction::new_with_name("rc_triple_func", |x: &i32| x * 3);
    let named_conditional = triple.when(|x: &i32| *x % 2 == 0);

    let named_debug_str = format!("{:?}", named_conditional);
    assert!(named_debug_str.contains("RcConditionalFunction"));
    assert!(named_debug_str.contains("name"));
    assert!(named_debug_str.contains("function"));
    assert!(named_debug_str.contains("predicate"));

    let named_display_str = format!("{}", named_conditional);
    assert!(named_display_str.starts_with("RcConditionalFunction("));
    assert!(named_display_str.contains("RcFunction(rc_triple_func)"));
    assert!(named_display_str.contains("RcPredicate"));
    assert!(named_display_str.ends_with(")"));
}

#[test]
fn test_arc_conditional_function_debug_display() {
    // Test Debug and Display for ArcConditionalFunction without name
    let double = ArcFunction::new(|x: &i32| x * 2);
    let conditional = double.when(|x: &i32| *x > 0);

    let debug_str = format!("{:?}", conditional);
    assert!(debug_str.contains("ArcConditionalFunction"));
    assert!(debug_str.contains("name"));
    assert!(debug_str.contains("function"));
    assert!(debug_str.contains("predicate"));

    let display_str = format!("{}", conditional);
    assert!(display_str.starts_with("ArcConditionalFunction("));
    assert!(display_str.contains("ArcFunction"));
    assert!(display_str.contains("ArcPredicate"));
    assert!(display_str.ends_with(")"));

    // Test Debug and Display for ArcConditionalFunction with name
    let triple = ArcFunction::new_with_name("arc_triple_func", |x: &i32| x * 3);
    let named_conditional = triple.when(|x: &i32| *x % 2 == 0);

    let named_debug_str = format!("{:?}", named_conditional);
    assert!(named_debug_str.contains("ArcConditionalFunction"));
    assert!(named_debug_str.contains("name"));
    assert!(named_debug_str.contains("function"));
    assert!(named_debug_str.contains("predicate"));

    let named_display_str = format!("{}", named_conditional);
    assert!(named_display_str.starts_with("ArcConditionalFunction("));
    assert!(named_display_str.contains("ArcFunction(arc_triple_func)"));
    assert!(named_display_str.contains("ArcPredicate"));
    assert!(named_display_str.ends_with(")"));
}

// ============================================================================
// RcConditionalFunction Clone Tests
// ============================================================================

#[test]
fn test_rc_conditional_function_clone() {
    let double = RcFunction::new(|x: &i32| x * 2);
    let conditional = double.when(|x: &i32| *x > 0);

    // Clone conditional function
    let conditional_clone = conditional.clone();

    // Both cloned conditional functions work properly
    let result1 = conditional.or_else(|x: &i32| -(*x));
    let result2 = conditional_clone.or_else(|x: &i32| x + 100);

    assert_eq!(result1.apply(&5), 10); // Condition met: 5 * 2
    assert_eq!(result1.apply(&-5), 5); // Condition not met: -(-5)
    assert_eq!(result2.apply(&5), 10); // Condition met: 5 * 2
    assert_eq!(result2.apply(&-5), 95); // Condition not met: -5 + 100
}

#[test]
fn test_rc_conditional_function_clone_multiple() {
    let triple = RcFunction::new(|x: &i32| x * 3);
    let conditional = triple.when(|x: &i32| *x % 2 == 0);

    // Create multiple clones
    let clone1 = conditional.clone();
    let clone2 = conditional.clone();
    let clone3 = conditional.clone();

    let result1 = conditional.or_else(|x: &i32| *x);
    let result2 = clone1.or_else(|x: &i32| *x);
    let result3 = clone2.or_else(|x: &i32| *x);
    let result4 = clone3.or_else(|x: &i32| *x);

    // All clones work properly
    assert_eq!(result1.apply(&4), 12); // Even number: 4 * 3
    assert_eq!(result2.apply(&4), 12);
    assert_eq!(result3.apply(&4), 12);
    assert_eq!(result4.apply(&4), 12);

    assert_eq!(result1.apply(&5), 5); // Odd number: 5
    assert_eq!(result2.apply(&5), 5);
    assert_eq!(result3.apply(&5), 5);
    assert_eq!(result4.apply(&5), 5);
}

// ============================================================================
// Name Preservation Tests for into_xxx and to_xxx Methods
// ============================================================================

#[test]
fn test_box_function_clear_name() {
    let mut function = BoxFunction::new_with_name("named_fn", |x: &i32| x * 2);
    assert_eq!(function.name(), Some("named_fn"));

    function.clear_name();
    assert_eq!(function.name(), None);
    assert_eq!(function.apply(&21), 42);
}

#[test]
fn test_box_function_set_name_same_value_keeps_storage() {
    let mut function = BoxFunction::new_with_name("stable_name", |x: &i32| x * 2);
    let ptr_before = function.name().expect("name should be initialized").as_ptr();

    function.set_name("stable_name");
    let ptr_after = function.name().expect("name should remain initialized").as_ptr();

    assert_eq!(function.name(), Some("stable_name"));
    assert_eq!(ptr_before, ptr_after);
}
