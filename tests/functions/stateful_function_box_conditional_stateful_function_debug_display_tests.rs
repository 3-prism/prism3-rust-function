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
fn test_box_conditional_stateful_function_debug_display() {
    // Test Debug and Display for BoxConditionalStatefulFunction without name

    let mut double = BoxStatefulFunction::new(move |x: &i32| x * 2);
    // Call apply to test the function
    assert_eq!(double.apply(&5), 10);

    let conditional = double.when(|x: &i32| *x > 0);

    let debug_str = format!("{:?}", conditional);
    assert!(debug_str.contains("BoxConditionalStatefulFunction"));
    assert!(debug_str.contains("name"));
    assert!(debug_str.contains("function"));
    assert!(debug_str.contains("predicate"));

    let display_str = format!("{}", conditional);
    assert!(display_str.starts_with("BoxConditionalStatefulFunction("));
    assert!(display_str.contains("BoxStatefulFunction"));
    assert!(display_str.contains("BoxPredicate"));
    assert!(display_str.ends_with(")"));

    // Test Debug and Display for BoxConditionalStatefulFunction with name
    let mut named_double = BoxStatefulFunction::new_with_name("stateful_double", |x: &i32| x * 2);
    // Call apply to test the function
    assert_eq!(named_double.apply(&3), 6);

    let named_conditional = named_double.when(|x: &i32| *x % 2 == 0);

    let named_debug_str = format!("{:?}", named_conditional);
    assert!(named_debug_str.contains("BoxConditionalStatefulFunction"));
    assert!(named_debug_str.contains("name"));
    assert!(named_debug_str.contains("function"));
    assert!(named_debug_str.contains("predicate"));

    let named_display_str = format!("{}", named_conditional);
    assert!(named_display_str.starts_with("BoxConditionalStatefulFunction("));
    assert!(named_display_str.contains("BoxStatefulFunction(stateful_double)"));
    assert!(named_display_str.contains("BoxPredicate"));
    assert!(named_display_str.ends_with(")"));
}

#[test]
fn test_rc_conditional_stateful_function_debug_display() {
    // Test Debug and Display for RcConditionalStatefulFunction without name

    let mut double = RcStatefulFunction::new(move |x: &i32| x * 2);
    // Call apply to test the function
    assert_eq!(double.apply(&5), 10);

    let conditional = double.when(|x: &i32| *x > 0);

    let debug_str = format!("{:?}", conditional);
    assert!(debug_str.contains("RcConditionalStatefulFunction"));
    assert!(debug_str.contains("name"));
    assert!(debug_str.contains("function"));
    assert!(debug_str.contains("predicate"));

    let display_str = format!("{}", conditional);
    assert!(display_str.starts_with("RcConditionalStatefulFunction("));
    assert!(display_str.contains("RcStatefulFunction"));
    assert!(display_str.contains("RcPredicate"));
    assert!(display_str.ends_with(")"));

    // Test Debug and Display for RcConditionalStatefulFunction with name
    let mut named_double = RcStatefulFunction::new_with_name("rc_stateful_double", |x: &i32| x * 2);
    // Call apply to test the function
    assert_eq!(named_double.apply(&3), 6);

    let named_conditional = named_double.when(|x: &i32| *x % 2 == 0);

    let named_debug_str = format!("{:?}", named_conditional);
    assert!(named_debug_str.contains("RcConditionalStatefulFunction"));
    assert!(named_debug_str.contains("name"));
    assert!(named_debug_str.contains("function"));
    assert!(named_debug_str.contains("predicate"));

    let named_display_str = format!("{}", named_conditional);
    assert!(named_display_str.starts_with("RcConditionalStatefulFunction("));
    assert!(named_display_str.contains("RcStatefulFunction(rc_stateful_double)"));
    assert!(named_display_str.contains("RcPredicate"));
    assert!(named_display_str.ends_with(")"));
}

#[test]
fn test_arc_conditional_stateful_function_debug_display() {
    // Test Debug and Display for ArcConditionalStatefulFunction without name

    let mut double = ArcStatefulFunction::new(move |x: &i32| x * 2);
    // Call apply to test the function
    assert_eq!(double.apply(&5), 10);

    let conditional = double.when(|x: &i32| *x > 0);

    let debug_str = format!("{:?}", conditional);
    assert!(debug_str.contains("ArcConditionalStatefulFunction"));
    assert!(debug_str.contains("name"));
    assert!(debug_str.contains("function"));
    assert!(debug_str.contains("predicate"));

    let display_str = format!("{}", conditional);
    assert!(display_str.starts_with("ArcConditionalStatefulFunction("));
    assert!(display_str.contains("ArcStatefulFunction"));
    assert!(display_str.contains("ArcPredicate"));
    assert!(display_str.ends_with(")"));

    // Test Debug and Display for ArcConditionalStatefulFunction with name
    let mut named_double = ArcStatefulFunction::new_with_name("arc_stateful_double", |x: &i32| x * 2);
    // Call apply to test the function
    assert_eq!(named_double.apply(&3), 6);

    let named_conditional = named_double.when(|x: &i32| *x % 2 == 0);

    let named_debug_str = format!("{:?}", named_conditional);
    assert!(named_debug_str.contains("ArcConditionalStatefulFunction"));
    assert!(named_debug_str.contains("name"));
    assert!(named_debug_str.contains("function"));
    assert!(named_debug_str.contains("predicate"));

    let named_display_str = format!("{}", named_conditional);
    assert!(named_display_str.starts_with("ArcConditionalStatefulFunction("));
    assert!(named_display_str.contains("ArcStatefulFunction(arc_stateful_double)"));
    assert!(named_display_str.contains("ArcPredicate"));
    assert!(named_display_str.ends_with(")"));
}

// ============================================================================
// StatefulFunction Trait Default Methods Tests - into_once, to_once
// ============================================================================
