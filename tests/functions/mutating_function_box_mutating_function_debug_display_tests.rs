// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Unit tests for MutatingFunction types (stateless Fn(&mut T) -> R)

use qubit_function::ArcMutatingFunction;
use qubit_function::BoxMutatingFunction;
use qubit_function::MutatingFunction;
use qubit_function::MutatingFunctionOnce;
use qubit_function::RcMutatingFunction;

// ============================================================================
// BoxMutatingFunction Tests
// ============================================================================

/// Test struct that implements MutatingFunction to test default methods
struct TestMutatingFunction {
    multiplier: i32,
}

impl TestMutatingFunction {
    fn new(multiplier: i32) -> Self {
        TestMutatingFunction { multiplier }
    }
}

impl MutatingFunction<i32, i32> for TestMutatingFunction {
    fn apply(&self, input: &mut i32) -> i32 {
        let old_value = *input;
        *input *= self.multiplier;
        old_value
    }
}

impl Clone for TestMutatingFunction {
    fn clone(&self) -> Self {
        TestMutatingFunction {
            multiplier: self.multiplier,
        }
    }
}

// ============================================================================
// MutatingFunction Debug and Display Tests
// ============================================================================

#[test]
fn test_box_mutating_function_debug_display() {
    // Test Debug and Display for BoxMutatingFunction without name
    let double = BoxMutatingFunction::new(|x: &mut i32| *x * 2);
    let debug_str = format!("{:?}", double);
    assert!(debug_str.contains("BoxMutatingFunction"));
    assert!(debug_str.contains("name"));
    assert!(debug_str.contains("function"));

    let display_str = format!("{}", double);
    assert_eq!(display_str, "BoxMutatingFunction");

    // Test Debug and Display for BoxMutatingFunction with name
    let named_double = BoxMutatingFunction::new_with_name("mutating_double", |x: &mut i32| *x * 2);
    let named_debug_str = format!("{:?}", named_double);
    assert!(named_debug_str.contains("BoxMutatingFunction"));
    assert!(named_debug_str.contains("name"));
    assert!(named_debug_str.contains("function"));

    let named_display_str = format!("{}", named_double);
    assert_eq!(named_display_str, "BoxMutatingFunction(mutating_double)");
}

#[test]
fn test_rc_mutating_function_debug_display() {
    // Test Debug and Display for RcMutatingFunction without name
    let double = RcMutatingFunction::new(|x: &mut i32| *x * 2);
    let debug_str = format!("{:?}", double);
    assert!(debug_str.contains("RcMutatingFunction"));
    assert!(debug_str.contains("name"));
    assert!(debug_str.contains("function"));

    let display_str = format!("{}", double);
    assert_eq!(display_str, "RcMutatingFunction");

    // Test Debug and Display for RcMutatingFunction with name
    let named_double = RcMutatingFunction::new_with_name("rc_mutating_double", |x: &mut i32| *x * 2);
    let named_debug_str = format!("{:?}", named_double);
    assert!(named_debug_str.contains("RcMutatingFunction"));
    assert!(named_debug_str.contains("name"));
    assert!(named_debug_str.contains("function"));

    let named_display_str = format!("{}", named_double);
    assert_eq!(named_display_str, "RcMutatingFunction(rc_mutating_double)");
}

#[test]
fn test_arc_mutating_function_debug_display() {
    // Test Debug and Display for ArcMutatingFunction without name
    let double = ArcMutatingFunction::new(|x: &mut i32| *x * 2);
    let debug_str = format!("{:?}", double);
    assert!(debug_str.contains("ArcMutatingFunction"));
    assert!(debug_str.contains("name"));
    assert!(debug_str.contains("function"));

    let display_str = format!("{}", double);
    assert_eq!(display_str, "ArcMutatingFunction");

    // Test Debug and Display for ArcMutatingFunction with name
    let named_double = ArcMutatingFunction::new_with_name("arc_mutating_double", |x: &mut i32| *x * 2);
    let named_debug_str = format!("{:?}", named_double);
    assert!(named_debug_str.contains("ArcMutatingFunction"));
    assert!(named_debug_str.contains("name"));
    assert!(named_debug_str.contains("function"));

    let named_display_str = format!("{}", named_double);
    assert_eq!(named_display_str, "ArcMutatingFunction(arc_mutating_double)");
}

// ============================================================================
// MutatingFunction Name Management Tests
// ============================================================================

#[test]
fn test_box_mutating_function_name_methods() {
    // Test new_with_name, name(), and set_name()
    let mut double = BoxMutatingFunction::new_with_name("box_mutating_func", |x: &mut i32| {
        *x *= 2;
        *x
    });

    // Test name() returns the initial name
    assert_eq!(double.name(), Some("box_mutating_func"));

    // Test set_name() changes the name
    double.set_name("modified_box_mutating");
    assert_eq!(double.name(), Some("modified_box_mutating"));

    // Test that function still works after name change
    let mut value = 5;
    assert_eq!(double.apply(&mut value), 10);
    assert_eq!(value, 10);
}

#[test]
fn test_rc_mutating_function_name_methods() {
    // Test new_with_name, name(), and set_name()
    let mut double = RcMutatingFunction::new_with_name("rc_mutating_func", |x: &mut i32| {
        *x *= 2;
        *x
    });

    // Test name() returns the initial name
    assert_eq!(double.name(), Some("rc_mutating_func"));

    // Test set_name() changes the name
    double.set_name("modified_rc_mutating");
    assert_eq!(double.name(), Some("modified_rc_mutating"));

    // Test that function still works after name change
    let mut value = 5;
    assert_eq!(double.apply(&mut value), 10);
    assert_eq!(value, 10);

    // Test cloning preserves name
    let cloned = double.clone();
    assert_eq!(cloned.name(), Some("modified_rc_mutating"));
    let mut value2 = 3;
    assert_eq!(cloned.apply(&mut value2), 6);
    assert_eq!(value2, 6);
}

#[test]
fn test_arc_mutating_function_name_methods() {
    // Test new_with_name, name(), and set_name()
    let mut double = ArcMutatingFunction::new_with_name("arc_mutating_func", |x: &mut i32| {
        *x *= 2;
        *x
    });

    // Test name() returns the initial name
    assert_eq!(double.name(), Some("arc_mutating_func"));

    // Test set_name() changes the name
    double.set_name("modified_arc_mutating");
    assert_eq!(double.name(), Some("modified_arc_mutating"));

    // Test that function still works after name change
    let mut value = 5;
    assert_eq!(double.apply(&mut value), 10);
    assert_eq!(value, 10);

    // Test cloning preserves name
    let cloned = double.clone();
    assert_eq!(cloned.name(), Some("modified_arc_mutating"));
    let mut value2 = 3;
    assert_eq!(cloned.apply(&mut value2), 6);
    assert_eq!(value2, 6);
}

// ============================================================================
// ConditionalMutatingFunction Debug and Display Tests
// ============================================================================

#[test]
fn test_box_conditional_mutating_function_debug_display() {
    // Test Debug and Display for BoxConditionalMutatingFunction without name
    let double = BoxMutatingFunction::new(|x: &mut i32| *x * 2);
    let conditional = double.when(|x: &i32| *x > 0);

    let debug_str = format!("{:?}", conditional);
    assert!(debug_str.contains("BoxConditionalMutatingFunction"));
    assert!(debug_str.contains("name"));
    assert!(debug_str.contains("function"));
    assert!(debug_str.contains("predicate"));

    let display_str = format!("{}", conditional);
    assert!(display_str.starts_with("BoxConditionalMutatingFunction("));
    assert!(display_str.contains("BoxMutatingFunction"));
    assert!(display_str.contains("BoxPredicate"));
    assert!(display_str.ends_with(")"));

    // Test Debug and Display for BoxConditionalMutatingFunction with name
    let triple = BoxMutatingFunction::new_with_name("triple_mutating_func", |x: &mut i32| *x * 3);
    let named_conditional = triple.when(|x: &i32| *x % 2 == 0);

    let named_debug_str = format!("{:?}", named_conditional);
    assert!(named_debug_str.contains("BoxConditionalMutatingFunction"));
    assert!(named_debug_str.contains("name"));
    assert!(named_debug_str.contains("function"));
    assert!(named_debug_str.contains("predicate"));

    let named_display_str = format!("{}", named_conditional);
    assert!(named_display_str.starts_with("BoxConditionalMutatingFunction("));
    assert!(named_display_str.contains("BoxMutatingFunction(triple_mutating_func)"));
    assert!(named_display_str.contains("BoxPredicate"));
    assert!(named_display_str.ends_with(")"));
}

#[test]
fn test_rc_conditional_mutating_function_debug_display() {
    // Test Debug and Display for RcConditionalMutatingFunction without name
    let double = RcMutatingFunction::new(|x: &mut i32| *x * 2);
    let conditional = double.when(|x: &i32| *x > 0);

    let debug_str = format!("{:?}", conditional);
    assert!(debug_str.contains("RcConditionalMutatingFunction"));
    assert!(debug_str.contains("name"));
    assert!(debug_str.contains("function"));
    assert!(debug_str.contains("predicate"));

    let display_str = format!("{}", conditional);
    assert!(display_str.starts_with("RcConditionalMutatingFunction("));
    assert!(display_str.contains("RcMutatingFunction"));
    assert!(display_str.contains("RcPredicate"));
    assert!(display_str.ends_with(")"));

    // Test Debug and Display for RcConditionalMutatingFunction with name
    let triple = RcMutatingFunction::new_with_name("rc_triple_mutating_func", |x: &mut i32| *x * 3);
    let named_conditional = triple.when(|x: &i32| *x % 2 == 0);

    let named_debug_str = format!("{:?}", named_conditional);
    assert!(named_debug_str.contains("RcConditionalMutatingFunction"));
    assert!(named_debug_str.contains("name"));
    assert!(named_debug_str.contains("function"));
    assert!(named_debug_str.contains("predicate"));

    let named_display_str = format!("{}", named_conditional);
    assert!(named_display_str.starts_with("RcConditionalMutatingFunction("));
    assert!(named_display_str.contains("RcMutatingFunction(rc_triple_mutating_func)"));
    assert!(named_display_str.contains("RcPredicate"));
    assert!(named_display_str.ends_with(")"));
}

#[test]
fn test_arc_conditional_mutating_function_debug_display() {
    // Test Debug and Display for ArcConditionalMutatingFunction without name
    let double = ArcMutatingFunction::new(|x: &mut i32| *x * 2);
    let conditional = double.when(|x: &i32| *x > 0);

    let debug_str = format!("{:?}", conditional);
    assert!(debug_str.contains("ArcConditionalMutatingFunction"));
    assert!(debug_str.contains("name"));
    assert!(debug_str.contains("function"));
    assert!(debug_str.contains("predicate"));

    let display_str = format!("{}", conditional);
    assert!(display_str.starts_with("ArcConditionalMutatingFunction("));
    assert!(display_str.contains("ArcMutatingFunction"));
    assert!(display_str.contains("ArcPredicate"));
    assert!(display_str.ends_with(")"));

    // Test Debug and Display for ArcConditionalMutatingFunction with name
    let triple = ArcMutatingFunction::new_with_name("arc_triple_mutating_func", |x: &mut i32| *x * 3);
    let named_conditional = triple.when(|x: &i32| *x % 2 == 0);

    let named_debug_str = format!("{:?}", named_conditional);
    assert!(named_debug_str.contains("ArcConditionalMutatingFunction"));
    assert!(named_debug_str.contains("name"));
    assert!(named_debug_str.contains("function"));
    assert!(named_debug_str.contains("predicate"));

    let named_display_str = format!("{}", named_conditional);
    assert!(named_display_str.starts_with("ArcConditionalMutatingFunction("));
    assert!(named_display_str.contains("ArcMutatingFunction(arc_triple_mutating_func)"));
    assert!(named_display_str.contains("ArcPredicate"));
    assert!(named_display_str.ends_with(")"));
}

// ============================================================================
// MutatingFunction Trait Default Methods Tests - into_once, to_once
// ============================================================================
