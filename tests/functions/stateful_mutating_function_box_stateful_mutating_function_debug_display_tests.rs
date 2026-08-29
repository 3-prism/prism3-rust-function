// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Unit tests for StatefulMutatingFunction types (stateful FnMut(&mut T) ->
//! R)

use std::cell::RefCell;
use std::rc::Rc;

use qubit_function::ArcStatefulMutatingFunction;
use qubit_function::BoxStatefulMutatingFunction;
use qubit_function::MutatingFunctionOnce;
use qubit_function::RcStatefulMutatingFunction;
use qubit_function::StatefulMutatingFunction;

// ============================================================================
// StatefulMutatingFunction Default Implementation Tests
// ============================================================================

/// Test struct that implements StatefulMutatingFunction to test default methods
struct TestStatefulMutatingFunction {
    multiplier: i32,
}

impl TestStatefulMutatingFunction {
    fn new(multiplier: i32) -> Self {
        TestStatefulMutatingFunction { multiplier }
    }
}

impl StatefulMutatingFunction<i32, i32> for TestStatefulMutatingFunction {
    fn apply(&mut self, input: &mut i32) -> i32 {
        let old_value = *input;
        *input *= self.multiplier;
        old_value
    }
}

impl Clone for TestStatefulMutatingFunction {
    fn clone(&self) -> Self {
        TestStatefulMutatingFunction {
            multiplier: self.multiplier,
        }
    }
}

// ============================================================================
// BoxStatefulMutatingFunction Tests
// ============================================================================

#[test]
fn test_box_stateful_mutating_function_debug_display() {
    // Test Debug and Display for BoxStatefulMutatingFunction without name

    let mut double = BoxStatefulMutatingFunction::new(move |x: &mut i32| *x * 2);
    // Call apply to use the counter variable
    let mut value1 = 5;
    let _result1 = double.apply(&mut value1);

    let debug_str = format!("{:?}", double);
    assert!(debug_str.contains("BoxStatefulMutatingFunction"));
    assert!(debug_str.contains("name"));
    assert!(debug_str.contains("function"));

    let display_str = format!("{}", double);
    assert_eq!(display_str, "BoxStatefulMutatingFunction");

    // Test Debug and Display for BoxStatefulMutatingFunction with name
    let mut named_double = BoxStatefulMutatingFunction::new_with_name("box_stateful_mutating", |x: &mut i32| *x * 2);
    // Call apply to ensure the function works
    let mut value2 = 3;
    let _result2 = named_double.apply(&mut value2);

    let named_debug_str = format!("{:?}", named_double);
    assert!(named_debug_str.contains("BoxStatefulMutatingFunction"));
    assert!(named_debug_str.contains("name"));
    assert!(named_debug_str.contains("function"));

    let named_display_str = format!("{}", named_double);
    assert_eq!(named_display_str, "BoxStatefulMutatingFunction(box_stateful_mutating)");
}

#[test]
fn test_rc_stateful_mutating_function_debug_display() {
    // Test Debug and Display for RcStatefulMutatingFunction without name

    let mut double = RcStatefulMutatingFunction::new(move |x: &mut i32| *x * 2);
    // Call apply to use the counter variable
    let mut value1 = 5;
    let _result1 = double.apply(&mut value1);

    let debug_str = format!("{:?}", double);
    assert!(debug_str.contains("RcStatefulMutatingFunction"));
    assert!(debug_str.contains("name"));
    assert!(debug_str.contains("function"));

    let display_str = format!("{}", double);
    assert_eq!(display_str, "RcStatefulMutatingFunction");

    // Test Debug and Display for RcStatefulMutatingFunction with name
    let mut named_double = RcStatefulMutatingFunction::new_with_name("rc_stateful_mutating", |x: &mut i32| *x * 2);
    // Call apply to ensure the function works
    let mut value2 = 3;
    let _result2 = named_double.apply(&mut value2);

    let named_debug_str = format!("{:?}", named_double);
    assert!(named_debug_str.contains("RcStatefulMutatingFunction"));
    assert!(named_debug_str.contains("name"));
    assert!(named_debug_str.contains("function"));

    let named_display_str = format!("{}", named_double);
    assert_eq!(named_display_str, "RcStatefulMutatingFunction(rc_stateful_mutating)");
}

#[test]
fn test_arc_stateful_mutating_function_debug_display() {
    // Test Debug and Display for ArcStatefulMutatingFunction without name

    let mut double = ArcStatefulMutatingFunction::new(move |x: &mut i32| *x * 2);
    // Call apply to use the counter variable
    let mut value1 = 5;
    let _result1 = double.apply(&mut value1);

    let debug_str = format!("{:?}", double);
    assert!(debug_str.contains("ArcStatefulMutatingFunction"));
    assert!(debug_str.contains("name"));
    assert!(debug_str.contains("function"));

    let display_str = format!("{}", double);
    assert_eq!(display_str, "ArcStatefulMutatingFunction");

    // Test Debug and Display for ArcStatefulMutatingFunction with name
    let mut named_double = ArcStatefulMutatingFunction::new_with_name("arc_stateful_mutating", |x: &mut i32| *x * 2);
    // Call apply to ensure the function works
    let mut value2 = 3;
    let _result2 = named_double.apply(&mut value2);

    let named_debug_str = format!("{:?}", named_double);
    assert!(named_debug_str.contains("ArcStatefulMutatingFunction"));
    assert!(named_debug_str.contains("name"));
    assert!(named_debug_str.contains("function"));

    let named_display_str = format!("{}", named_double);
    assert_eq!(named_display_str, "ArcStatefulMutatingFunction(arc_stateful_mutating)");
}

// ============================================================================
// StatefulMutatingFunction Name Management Tests
// ============================================================================

#[test]
fn test_box_stateful_mutating_function_name_methods() {
    // Test new_with_name, name(), and set_name()

    let mut double = BoxStatefulMutatingFunction::new_with_name("box_stateful_mutating_func", move |x: &mut i32| {
        *x *= 2;
        *x
    });

    // Test name() returns the initial name
    assert_eq!(double.name(), Some("box_stateful_mutating_func"));

    // Test set_name() changes the name
    double.set_name("modified_box_stateful_mutating");
    assert_eq!(double.name(), Some("modified_box_stateful_mutating"));

    // Test that function still works after name change
    let mut value = 5;
    assert_eq!(double.apply(&mut value), 10);
    assert_eq!(value, 10);
}

#[test]
fn test_rc_stateful_mutating_function_name_methods() {
    // Test new_with_name, name(), and set_name()

    let mut double = RcStatefulMutatingFunction::new_with_name("rc_stateful_mutating_func", move |x: &mut i32| {
        *x *= 2;
        *x
    });

    // Test name() returns the initial name
    assert_eq!(double.name(), Some("rc_stateful_mutating_func"));

    // Test set_name() changes the name
    double.set_name("modified_rc_stateful_mutating");
    assert_eq!(double.name(), Some("modified_rc_stateful_mutating"));

    // Test that function still works after name change
    let mut value = 5;
    assert_eq!(double.apply(&mut value), 10);
    assert_eq!(value, 10);

    // Test cloning preserves name
    let mut cloned = double.clone();
    assert_eq!(cloned.name(), Some("modified_rc_stateful_mutating"));
    let mut value2 = 3;
    assert_eq!(cloned.apply(&mut value2), 6);
    assert_eq!(value2, 6);
}

#[test]
fn test_arc_stateful_mutating_function_name_methods() {
    // Test new_with_name, name(), and set_name()

    let mut double = ArcStatefulMutatingFunction::new_with_name("arc_stateful_mutating_func", move |x: &mut i32| {
        *x *= 2;
        *x
    });

    // Test name() returns the initial name
    assert_eq!(double.name(), Some("arc_stateful_mutating_func"));

    // Test set_name() changes the name
    double.set_name("modified_arc_stateful_mutating");
    assert_eq!(double.name(), Some("modified_arc_stateful_mutating"));

    // Test that function still works after name change
    let mut value = 5;
    assert_eq!(double.apply(&mut value), 10);
    assert_eq!(value, 10);

    // Test cloning preserves name
    let mut cloned = double.clone();
    assert_eq!(cloned.name(), Some("modified_arc_stateful_mutating"));
    let mut value2 = 3;
    assert_eq!(cloned.apply(&mut value2), 6);
    assert_eq!(value2, 6);
}

// ============================================================================
// ConditionalStatefulMutatingFunction Debug and Display Tests
// ============================================================================

#[test]
fn test_box_conditional_stateful_mutating_function_debug_display() {
    // Test Debug and Display for BoxConditionalStatefulMutatingFunction without
    // name

    let mut double = BoxStatefulMutatingFunction::new(move |x: &mut i32| {
        *x *= 2;
        *x
    });
    // Call apply to use the counter variable
    let mut test_val = 5;
    assert_eq!(double.apply(&mut test_val), 10);
    assert_eq!(test_val, 10);

    let conditional = double.when(|x: &i32| *x > 0);

    let debug_str = format!("{:?}", conditional);
    assert!(debug_str.contains("BoxConditionalStatefulMutatingFunction"));
    assert!(debug_str.contains("name"));
    assert!(debug_str.contains("function"));
    assert!(debug_str.contains("predicate"));

    let display_str = format!("{}", conditional);
    assert!(display_str.starts_with("BoxConditionalStatefulMutatingFunction("));
    assert!(display_str.contains("BoxStatefulMutatingFunction"));
    assert!(display_str.contains("BoxPredicate"));
    assert!(display_str.ends_with(")"));

    // Test Debug and Display for BoxConditionalStatefulMutatingFunction with
    // name
    let mut named_double = BoxStatefulMutatingFunction::new_with_name("stateful_mutating_double", |x: &mut i32| {
        *x *= 2;
        *x
    });
    // Call apply to ensure the function works
    let mut test_val2 = 3;
    assert_eq!(named_double.apply(&mut test_val2), 6);
    assert_eq!(test_val2, 6);

    let named_conditional = named_double.when(|x: &i32| *x % 2 == 0);

    let named_debug_str = format!("{:?}", named_conditional);
    assert!(named_debug_str.contains("BoxConditionalStatefulMutatingFunction"));
    assert!(named_debug_str.contains("name"));
    assert!(named_debug_str.contains("function"));
    assert!(named_debug_str.contains("predicate"));

    let named_display_str = format!("{}", named_conditional);
    assert!(named_display_str.starts_with("BoxConditionalStatefulMutatingFunction("));
    assert!(named_display_str.contains("BoxStatefulMutatingFunction(stateful_mutating_double)"));
    assert!(named_display_str.contains("BoxPredicate"));
    assert!(named_display_str.ends_with(")"));
}

#[test]
fn test_rc_conditional_stateful_mutating_function_debug_display() {
    // Test Debug and Display for RcConditionalStatefulMutatingFunction without
    // name

    let mut double = RcStatefulMutatingFunction::new(move |x: &mut i32| {
        *x *= 2;
        *x
    });
    // Call apply to use the counter variable
    let mut test_val = 5;
    assert_eq!(double.apply(&mut test_val), 10);
    assert_eq!(test_val, 10);

    let conditional = double.when(|x: &i32| *x > 0);

    let debug_str = format!("{:?}", conditional);
    assert!(debug_str.contains("RcConditionalStatefulMutatingFunction"));
    assert!(debug_str.contains("name"));
    assert!(debug_str.contains("function"));
    assert!(debug_str.contains("predicate"));

    let display_str = format!("{}", conditional);
    assert!(display_str.starts_with("RcConditionalStatefulMutatingFunction("));
    assert!(display_str.contains("RcStatefulMutatingFunction"));
    assert!(display_str.contains("RcPredicate"));
    assert!(display_str.ends_with(")"));

    // Test Debug and Display for RcConditionalStatefulMutatingFunction with
    // name
    let mut named_double = RcStatefulMutatingFunction::new_with_name("rc_stateful_mutating_double", |x: &mut i32| {
        *x *= 2;
        *x
    });
    // Call apply to ensure the function works
    let mut test_val2 = 3;
    assert_eq!(named_double.apply(&mut test_val2), 6);
    assert_eq!(test_val2, 6);

    let named_conditional = named_double.when(|x: &i32| *x % 2 == 0);

    let named_debug_str = format!("{:?}", named_conditional);
    assert!(named_debug_str.contains("RcConditionalStatefulMutatingFunction"));
    assert!(named_debug_str.contains("name"));
    assert!(named_debug_str.contains("function"));
    assert!(named_debug_str.contains("predicate"));

    let named_display_str = format!("{}", named_conditional);
    assert!(named_display_str.starts_with("RcConditionalStatefulMutatingFunction("));
    assert!(named_display_str.contains("RcStatefulMutatingFunction(rc_stateful_mutating_double)"));
    assert!(named_display_str.contains("RcPredicate"));
    assert!(named_display_str.ends_with(")"));
}

#[test]
fn test_arc_conditional_stateful_mutating_function_debug_display() {
    // Test Debug and Display for ArcConditionalStatefulMutatingFunction without
    // name

    let mut double = ArcStatefulMutatingFunction::new(move |x: &mut i32| {
        *x *= 2;
        *x
    });
    // Call apply to use the counter variable
    let mut test_val = 5;
    assert_eq!(double.apply(&mut test_val), 10);
    assert_eq!(test_val, 10);

    let conditional = double.when(|x: &i32| *x > 0);

    let debug_str = format!("{:?}", conditional);
    assert!(debug_str.contains("ArcConditionalStatefulMutatingFunction"));
    assert!(debug_str.contains("name"));
    assert!(debug_str.contains("function"));
    assert!(debug_str.contains("predicate"));

    let display_str = format!("{}", conditional);
    assert!(display_str.starts_with("ArcConditionalStatefulMutatingFunction("));
    assert!(display_str.contains("ArcStatefulMutatingFunction"));
    assert!(display_str.contains("ArcPredicate"));
    assert!(display_str.ends_with(")"));

    // Test Debug and Display for ArcConditionalStatefulMutatingFunction with
    // name
    let mut named_double = ArcStatefulMutatingFunction::new_with_name("arc_stateful_mutating_double", |x: &mut i32| {
        *x *= 2;
        *x
    });
    // Call apply to ensure the function works
    let mut test_val2 = 3;
    assert_eq!(named_double.apply(&mut test_val2), 6);
    assert_eq!(test_val2, 6);

    let named_conditional = named_double.when(|x: &i32| *x % 2 == 0);

    let named_debug_str = format!("{:?}", named_conditional);
    assert!(named_debug_str.contains("ArcConditionalStatefulMutatingFunction"));
    assert!(named_debug_str.contains("name"));
    assert!(named_debug_str.contains("function"));
    assert!(named_debug_str.contains("predicate"));

    let named_display_str = format!("{}", named_conditional);
    assert!(named_display_str.starts_with("ArcConditionalStatefulMutatingFunction("));
    assert!(named_display_str.contains("ArcStatefulMutatingFunction(arc_stateful_mutating_double)"));
    assert!(named_display_str.contains("ArcPredicate"));
    assert!(named_display_str.ends_with(")"));
}

// ============================================================================
// StatefulMutatingFunction Trait Default Methods Tests - into_once, to_once
// ============================================================================
