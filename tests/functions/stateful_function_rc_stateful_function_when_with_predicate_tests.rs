// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Comprehensive tests for StatefulFunction trait and its implementations

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::Mutex;

use qubit_function::ArcPredicate;
use qubit_function::ArcStatefulFunction;
use qubit_function::BoxPredicate;
use qubit_function::BoxStatefulFunction;
use qubit_function::FunctionOnce;
use qubit_function::RcPredicate;
use qubit_function::RcStatefulFunction;
use qubit_function::StatefulFunction;

// ============================================================================
// StatefulFunction Trait Tests - Core Functionality
// ============================================================================

/// Custom struct for testing StatefulFunction trait default implementations
#[derive(Clone)]
struct CustomStatefulFunction {
    multiplier: i32,
}

impl StatefulFunction<i32, i32> for CustomStatefulFunction {
    fn apply(&mut self, input: &i32) -> i32 {
        self.multiplier += 1;
        input * self.multiplier
    }
}

// ============================================================================
// ArcConditionalStatefulFunction Clone Tests
// ============================================================================

#[test]
fn test_rc_stateful_function_when_with_predicate() {
    // Test when with RcPredicate

    let counter = Rc::new(RefCell::new(0));
    let counter_clone = Rc::clone(&counter);
    let mut func = RcStatefulFunction::new(move |x: &i32| {
        let mut current = counter_clone.borrow_mut();
        let result = x * *current;
        *current += 1;
        result
    })
    .when(RcPredicate::new(|x: &i32| *x > 0))
    .or_else(|x: &i32| -(*x));

    assert_eq!(func.apply(&10), 0); // 10 > 0, apply * 0
    assert_eq!(func.apply(&-5), 5); // -5 <= 0, apply negate
}

// ============================================================================
// RcStatefulFunction Tests - Type Conversions
// ============================================================================

// ============================================================================
// Edge Cases and Boundary Tests
// ============================================================================

#[test]
fn test_stateful_function_with_zero() {
    // Test stateful function with zero input

    let counter = Rc::new(RefCell::new(0));
    let counter_clone = Rc::clone(&counter);
    let mut func = BoxStatefulFunction::new(move |x: &i32| {
        let current = *counter_clone.borrow();
        *counter_clone.borrow_mut() += 1;
        x + current
    });
    assert_eq!(func.apply(&0), 0);
    assert_eq!(func.apply(&0), 1);
}

#[test]
fn test_stateful_function_with_negative() {
    // Test stateful function with negative input

    let counter = Rc::new(RefCell::new(0));
    let counter_clone = Rc::clone(&counter);
    let mut func = BoxStatefulFunction::new(move |x: &i32| {
        let current = *counter_clone.borrow();
        *counter_clone.borrow_mut() += 1;
        x * current
    });
    assert_eq!(func.apply(&-10), 0);
    assert_eq!(func.apply(&-10), -10);
}

#[test]
fn test_stateful_function_accumulator() {
    // Test stateful function as accumulator
    let mut sum = 0;
    let mut func = BoxStatefulFunction::new(move |x: &i32| {
        sum += *x;
        sum
    });
    assert_eq!(func.apply(&1), 1);
    assert_eq!(func.apply(&2), 3);
    assert_eq!(func.apply(&3), 6);
    assert_eq!(func.apply(&4), 10);
}

#[test]
fn test_stateful_function_with_string() {
    // Test stateful function with String type
    let mut buffer = String::new();
    let mut func = BoxStatefulFunction::new(move |s: &String| {
        buffer.push_str(s);
        buffer.clone()
    });
    assert_eq!(func.apply(&String::from("Hello")), "Hello");
    assert_eq!(func.apply(&String::from(" ")), "Hello ");
    assert_eq!(func.apply(&String::from("World")), "Hello World");
}

#[test]
fn test_stateful_function_with_vec() {
    // Test stateful function with Vec type
    let mut history = Vec::new();
    let mut func = BoxStatefulFunction::new(move |x: &i32| {
        history.push(*x);
        history.len()
    });
    assert_eq!(func.apply(&1), 1);
    assert_eq!(func.apply(&2), 2);
    assert_eq!(func.apply(&3), 3);
}

#[test]
fn test_stateful_function_counter() {
    // Test stateful function as simple counter
    let mut count = 0;
    let mut func = BoxStatefulFunction::new(move |_x: &i32| {
        count += 1;
        count
    });
    assert_eq!(func.apply(&0), 1);
    assert_eq!(func.apply(&0), 2);
    assert_eq!(func.apply(&0), 3);
}

#[test]
fn test_stateful_function_toggle() {
    // Test stateful function as toggle
    let mut toggle = false;
    let mut func = BoxStatefulFunction::new(move |x: &i32| {
        toggle = !toggle;
        if toggle { *x } else { -*x }
    });
    assert_eq!(func.apply(&5), 5);
    assert_eq!(func.apply(&5), -5);
    assert_eq!(func.apply(&5), 5);
}

// ============================================================================
// Concrete wrapper composition tests
// ============================================================================

// ============================================================================
// Complex State Management Tests
// ============================================================================

#[test]
fn test_stateful_function_with_multiple_state() {
    // Test stateful function with multiple state variables
    let mut count = 0;
    let mut sum = 0;
    let mut func = BoxStatefulFunction::new(move |x: &i32| {
        count += 1;
        sum += *x;
        (count, sum)
    });
    assert_eq!(func.apply(&10), (1, 10));
    assert_eq!(func.apply(&20), (2, 30));
    assert_eq!(func.apply(&30), (3, 60));
}

#[test]
fn test_stateful_function_with_option_state() {
    // Test stateful function with Option state
    let mut last_value: Option<i32> = None;
    let mut func = BoxStatefulFunction::new(move |x: &i32| {
        let result = last_value.unwrap_or(0) + *x;
        last_value = Some(*x);
        result
    });
    assert_eq!(func.apply(&10), 10);
    assert_eq!(func.apply(&20), 30);
    assert_eq!(func.apply(&30), 50);
}

// ============================================================================
// Custom Struct Tests - StatefulFunction Default Implementation
// ============================================================================

#[test]
fn test_arc_conditional_stateful_function_clone() {
    // Test that ArcConditionalStatefulFunction can be cloned
    let counter = Arc::new(Mutex::new(0));
    let counter_clone = counter.clone();
    let conditional = ArcStatefulFunction::new(move |x: &i32| {
        *counter_clone.lock().expect("mutex should not be poisoned") += 1;
        x * 2
    })
    .when(|x: &i32| *x > 10);

    // Clone the conditional function before calling or_else
    let clone1 = conditional.clone();
    let clone2 = conditional.clone();

    // Convert to complete functions using or_else
    let mut func = conditional.or_else(|x: &i32| x + 1);
    let mut func_clone1 = clone1.or_else(|x: &i32| x + 1);
    let mut func_clone2 = clone2.or_else(|x: &i32| x + 1);

    // Test that all instances work independently but share the same counter
    assert_eq!(func.apply(&15), 30); // 15 > 10, apply * 2
    assert_eq!(*counter.lock().expect("mutex should not be poisoned"), 1);

    assert_eq!(func_clone1.apply(&20), 40); // 20 > 10, apply * 2
    assert_eq!(*counter.lock().expect("mutex should not be poisoned"), 2);

    assert_eq!(func_clone2.apply(&5), 6); // 5 <= 10, apply + 1
    assert_eq!(*counter.lock().expect("mutex should not be poisoned"), 2); // Counter not incremented

    assert_eq!(func_clone2.apply(&12), 24); // 12 > 10, apply * 2
    assert_eq!(*counter.lock().expect("mutex should not be poisoned"), 3);
}

// ============================================================================
// RcConditionalStatefulFunction Clone Tests
// ============================================================================

#[test]
fn test_rc_conditional_stateful_function_clone() {
    // Test that RcConditionalStatefulFunction can be cloned
    let counter = Rc::new(RefCell::new(0));
    let counter_clone = counter.clone();
    let conditional = RcStatefulFunction::new(move |x: &i32| {
        *counter_clone.borrow_mut() += 1;
        x * 2
    })
    .when(|x: &i32| *x > 10);

    // Clone the conditional function before calling or_else
    let clone1 = conditional.clone();
    let clone2 = conditional.clone();

    // Convert to complete functions using or_else
    let mut func = conditional.or_else(|x: &i32| x + 1);
    let mut func_clone1 = clone1.or_else(|x: &i32| x + 1);
    let mut func_clone2 = clone2.or_else(|x: &i32| x + 1);

    // Test that all instances work independently but share the same counter
    assert_eq!(func.apply(&15), 30); // 15 > 10, apply * 2
    assert_eq!(*counter.borrow(), 1);

    assert_eq!(func_clone1.apply(&20), 40); // 20 > 10, apply * 2
    assert_eq!(*counter.borrow(), 2);

    assert_eq!(func_clone2.apply(&5), 6); // 5 <= 10, apply + 1
    assert_eq!(*counter.borrow(), 2); // Counter not incremented

    assert_eq!(func_clone2.apply(&12), 24); // 12 > 10, apply * 2
    assert_eq!(*counter.borrow(), 3);
}

// ============================================================================
// StatefulFunction Debug and Display Tests
// ============================================================================

#[test]
fn test_box_stateful_function_debug_display() {
    // Test Debug and Display for BoxStatefulFunction without name

    let mut double = BoxStatefulFunction::new(move |x: &i32| x * 2);
    // Call apply to test the function
    assert_eq!(double.apply(&5), 10);

    let debug_str = format!("{:?}", double);
    assert!(debug_str.contains("BoxStatefulFunction"));
    assert!(debug_str.contains("name"));
    assert!(debug_str.contains("function"));

    let display_str = format!("{}", double);
    assert_eq!(display_str, "BoxStatefulFunction");

    // Test Debug and Display for BoxStatefulFunction with name
    let mut named_double = BoxStatefulFunction::new_with_name("stateful_double", |x: &i32| x * 2);
    // Call apply to test the function
    assert_eq!(named_double.apply(&3), 6);

    let named_debug_str = format!("{:?}", named_double);
    assert!(named_debug_str.contains("BoxStatefulFunction"));
    assert!(named_debug_str.contains("name"));
    assert!(named_debug_str.contains("function"));

    let named_display_str = format!("{}", named_double);
    assert_eq!(named_display_str, "BoxStatefulFunction(stateful_double)");
}

#[test]
fn test_rc_stateful_function_debug_display() {
    // Test Debug and Display for RcStatefulFunction without name

    let mut double = RcStatefulFunction::new(move |x: &i32| x * 2);
    // Call apply to test the function
    assert_eq!(double.apply(&5), 10);

    let debug_str = format!("{:?}", double);
    assert!(debug_str.contains("RcStatefulFunction"));
    assert!(debug_str.contains("name"));
    assert!(debug_str.contains("function"));

    let display_str = format!("{}", double);
    assert_eq!(display_str, "RcStatefulFunction");

    // Test Debug and Display for RcStatefulFunction with name
    let mut named_double = RcStatefulFunction::new_with_name("rc_stateful_double", |x: &i32| x * 2);
    // Call apply to test the function
    assert_eq!(named_double.apply(&3), 6);

    let named_debug_str = format!("{:?}", named_double);
    assert!(named_debug_str.contains("RcStatefulFunction"));
    assert!(named_debug_str.contains("name"));
    assert!(named_debug_str.contains("function"));

    let named_display_str = format!("{}", named_double);
    assert_eq!(named_display_str, "RcStatefulFunction(rc_stateful_double)");
}

#[test]
fn test_arc_stateful_function_debug_display() {
    // Test Debug and Display for ArcStatefulFunction without name

    let mut double = ArcStatefulFunction::new(move |x: &i32| x * 2);
    // Call apply to test the function
    assert_eq!(double.apply(&5), 10);

    let debug_str = format!("{:?}", double);
    assert!(debug_str.contains("ArcStatefulFunction"));
    assert!(debug_str.contains("name"));
    assert!(debug_str.contains("function"));

    let display_str = format!("{}", double);
    assert_eq!(display_str, "ArcStatefulFunction");

    // Test Debug and Display for ArcStatefulFunction with name
    let mut named_double = ArcStatefulFunction::new_with_name("arc_stateful_double", |x: &i32| x * 2);
    // Call apply to test the function
    assert_eq!(named_double.apply(&3), 6);

    let named_debug_str = format!("{:?}", named_double);
    assert!(named_debug_str.contains("ArcStatefulFunction"));
    assert!(named_debug_str.contains("name"));
    assert!(named_debug_str.contains("function"));

    let named_display_str = format!("{}", named_double);
    assert_eq!(named_display_str, "ArcStatefulFunction(arc_stateful_double)");
}

// ============================================================================
// StatefulFunction Name Management Tests
// ============================================================================

#[test]
fn test_box_stateful_function_name_methods() {
    // Test new_with_name, name(), and set_name()

    let mut double = BoxStatefulFunction::new_with_name("box_stateful_func", move |x: &i32| x * 2);

    // Test name() returns the initial name
    assert_eq!(double.name(), Some("box_stateful_func"));

    // Test set_name() changes the name
    double.set_name("modified_box_stateful");
    assert_eq!(double.name(), Some("modified_box_stateful"));

    // Test that function still works after name change
    assert_eq!(double.apply(&5), 10);
}

#[test]
fn test_rc_stateful_function_name_methods() {
    // Test new_with_name, name(), and set_name()

    let mut double = RcStatefulFunction::new_with_name("rc_stateful_func", move |x: &i32| x * 2);

    // Test name() returns the initial name
    assert_eq!(double.name(), Some("rc_stateful_func"));

    // Test set_name() changes the name
    double.set_name("modified_rc_stateful");
    assert_eq!(double.name(), Some("modified_rc_stateful"));

    // Test that function still works after name change
    assert_eq!(double.apply(&5), 10);

    // Test cloning preserves name
    let mut cloned = double.clone();
    assert_eq!(cloned.name(), Some("modified_rc_stateful"));
    assert_eq!(cloned.apply(&3), 6);
}

#[test]
fn test_arc_stateful_function_name_methods() {
    // Test new_with_name, name(), and set_name()

    let mut double = ArcStatefulFunction::new_with_name("arc_stateful_func", move |x: &i32| x * 2);

    // Test name() returns the initial name
    assert_eq!(double.name(), Some("arc_stateful_func"));

    // Test set_name() changes the name
    double.set_name("modified_arc_stateful");
    assert_eq!(double.name(), Some("modified_arc_stateful"));

    // Test that function still works after name change
    assert_eq!(double.apply(&5), 10);

    // Test cloning preserves name
    let mut cloned = double.clone();
    assert_eq!(cloned.name(), Some("modified_arc_stateful"));
    assert_eq!(cloned.apply(&3), 6);
}

// ============================================================================
// ConditionalStatefulFunction Debug and Display Tests
// ============================================================================
