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
fn test_bi_mutating_function_trait_apply() {
    // Test that BiMutatingFunction trait's apply method works correctly
    let swap_sum = |x: &mut i32, y: &mut i32| {
        std::mem::swap(&mut *x, &mut *y);
        *x + *y
    };

    let mut a = 20;
    let mut b = 22;
    assert_eq!(swap_sum.apply(&mut a, &mut b), 42);
    assert_eq!(a, 22);
    assert_eq!(b, 20);
}

#[test]
fn test_bi_mutating_function_trait_apply_with_complex_types() {
    let modify = |a: &mut TestStruct, b: &mut TestStruct| a.modify(b);

    let mut s1 = TestStruct::new(10);
    let mut s2 = TestStruct::new(5);
    let result = modify.apply(&mut s1, &mut s2);

    assert_eq!(result, 25); // (10+5) + (5*2) = 15 + 10 = 25
    assert_eq!(s1.value, 15);
    assert_eq!(s2.value, 10);
}

// ============================================================================
// BoxBiMutatingFunction Tests
// ============================================================================

#[test]
fn test_box_bi_mutating_function_new() {
    let swap_sum = BoxBiMutatingFunction::new(|x: &mut i32, y: &mut i32| {
        std::mem::swap(&mut *x, &mut *y);
        *x + *y
    });
    let mut a = 10;
    let mut b = 15;
    assert_eq!(swap_sum.apply(&mut a, &mut b), 25);
    assert_eq!(a, 15);
    assert_eq!(b, 10);
}

#[test]
fn test_box_bi_mutating_function_new_with_name() {
    let swap_sum = BoxBiMutatingFunction::new_with_name("swap_and_sum", |x: &mut i32, y: &mut i32| {
        std::mem::swap(&mut *x, &mut *y);
        *x + *y
    });
    assert_eq!(swap_sum.name(), Some("swap_and_sum"));
    let mut a = 10;
    let mut b = 15;
    assert_eq!(swap_sum.apply(&mut a, &mut b), 25);
}

#[test]
fn test_box_bi_mutating_function_new_with_optional_name() {
    let swap_sum = BoxBiMutatingFunction::new_with_optional_name(
        |x: &mut i32, y: &mut i32| {
            std::mem::swap(&mut *x, &mut *y);
            *x + *y
        },
        Some("test_function".to_string()),
    );
    assert_eq!(swap_sum.name(), Some("test_function"));

    let no_name = BoxBiMutatingFunction::new_with_optional_name(|x: &mut i32, y: &mut i32| *x + *y, None);
    assert_eq!(no_name.name(), None);
}

#[test]
fn test_box_bi_mutating_function_name_and_set_name() {
    let mut swap_sum = BoxBiMutatingFunction::new(|x: &mut i32, y: &mut i32| {
        std::mem::swap(&mut *x, &mut *y);
        *x + *y
    });

    assert_eq!(swap_sum.name(), None);
    swap_sum.set_name("modified_name");
    assert_eq!(swap_sum.name(), Some("modified_name"));
    swap_sum.set_name("another_name");
    assert_eq!(swap_sum.name(), Some("another_name"));
}

#[test]
fn test_box_bi_mutating_function_constant() {
    let constant = BoxBiMutatingFunction::constant(42);
    let mut a = 1;
    let mut b = 2;
    assert_eq!(constant.apply(&mut a, &mut b), 42);

    let mut c = 100;
    let mut d = 200;
    assert_eq!(constant.apply(&mut c, &mut d), 42);
}

#[test]
fn test_box_bi_mutating_function_debug_display() {
    let swap_sum = BoxBiMutatingFunction::new(|x: &mut i32, y: &mut i32| {
        std::mem::swap(&mut *x, &mut *y);
        *x + *y
    });

    let debug_str = format!("{:?}", swap_sum);
    assert!(debug_str.contains("BoxBiMutatingFunction"));

    let display_str = format!("{}", swap_sum);
    assert!(display_str.contains("BoxBiMutatingFunction"));
}

#[test]
fn test_box_bi_mutating_function_with_strings() {
    let append = BoxBiMutatingFunction::new(append_strings);
    let mut s1 = "hello".to_string();
    let mut s2 = "world".to_string();

    let result = append.apply(&mut s1, &mut s2);
    assert_eq!(result, 14 + 13); // "hello_modified".len() + "world_changed".len()
    assert_eq!(s1, "hello_modified");
    assert_eq!(s2, "world_changed");
}

#[test]
fn test_box_bi_mutating_function_with_structs() {
    let modify = BoxBiMutatingFunction::new(modify_structs);
    let mut s1 = TestStruct::new(10);
    let mut s2 = TestStruct::new(5);

    let result = modify.apply(&mut s1, &mut s2);
    assert_eq!(result, 25); // (10+5) + (5*2) = 15 + 10 = 25
    assert_eq!(s1.value, 15);
    assert_eq!(s2.value, 10);
}

// ============================================================================
// RcBiMutatingFunction Tests
// ============================================================================

#[test]
fn test_rc_bi_mutating_function_new() {
    let swap_sum = RcBiMutatingFunction::new(|x: &mut i32, y: &mut i32| {
        std::mem::swap(&mut *x, &mut *y);
        *x + *y
    });
    let mut a = 10;
    let mut b = 15;
    assert_eq!(swap_sum.apply(&mut a, &mut b), 25);
    assert_eq!(a, 15);
    assert_eq!(b, 10);
}

#[test]
fn test_rc_bi_mutating_function_clone() {
    let original = RcBiMutatingFunction::new(|x: &mut i32, y: &mut i32| {
        std::mem::swap(&mut *x, &mut *y);
        *x + *y
    });

    let cloned = original.clone();

    let mut a = 10;
    let mut b = 15;
    assert_eq!(original.apply(&mut a, &mut b), 25);
    assert_eq!(a, 15);
    assert_eq!(b, 10);

    let mut c = 20;
    let mut d = 25;
    assert_eq!(cloned.apply(&mut c, &mut d), 45);
    assert_eq!(c, 25);
    assert_eq!(d, 20);
}

#[test]
fn test_rc_bi_mutating_function_name_and_set_name() {
    let mut swap_sum = RcBiMutatingFunction::new_with_name("rc_function", |x: &mut i32, y: &mut i32| {
        std::mem::swap(&mut *x, &mut *y);
        *x + *y
    });

    assert_eq!(swap_sum.name(), Some("rc_function"));
    swap_sum.set_name("modified_rc");
    assert_eq!(swap_sum.name(), Some("modified_rc"));
}

#[test]
fn test_rc_bi_mutating_function_constant() {
    let constant = RcBiMutatingFunction::constant(99);
    let mut a = 1;
    let mut b = 2;
    assert_eq!(constant.apply(&mut a, &mut b), 99);

    let cloned = constant.clone();
    let mut c = 10;
    let mut d = 20;
    assert_eq!(cloned.apply(&mut c, &mut d), 99);
}

#[test]
fn test_rc_bi_mutating_function_debug_display() {
    let swap_sum = RcBiMutatingFunction::new(|x: &mut i32, y: &mut i32| {
        std::mem::swap(&mut *x, &mut *y);
        *x + *y
    });

    let debug_str = format!("{:?}", swap_sum);
    assert!(debug_str.contains("RcBiMutatingFunction"));

    let display_str = format!("{}", swap_sum);
    assert!(display_str.contains("RcBiMutatingFunction"));
}

// ============================================================================
// ArcBiMutatingFunction Tests
// ============================================================================

#[test]
fn test_arc_bi_mutating_function_new() {
    let swap_sum = ArcBiMutatingFunction::new(|x: &mut i32, y: &mut i32| {
        std::mem::swap(&mut *x, &mut *y);
        *x + *y
    });
    let mut a = 10;
    let mut b = 15;
    assert_eq!(swap_sum.apply(&mut a, &mut b), 25);
    assert_eq!(a, 15);
    assert_eq!(b, 10);
}

#[test]
fn test_arc_bi_mutating_function_clone() {
    let original = ArcBiMutatingFunction::new(|x: &mut i32, y: &mut i32| {
        std::mem::swap(&mut *x, &mut *y);
        *x + *y
    });

    let cloned = original.clone();

    let mut a = 10;
    let mut b = 15;
    assert_eq!(original.apply(&mut a, &mut b), 25);
    assert_eq!(a, 15);
    assert_eq!(b, 10);

    let mut c = 20;
    let mut d = 25;
    assert_eq!(cloned.apply(&mut c, &mut d), 45);
    assert_eq!(c, 25);
    assert_eq!(d, 20);
}

#[test]
fn test_arc_bi_mutating_function_thread_safety() {
    use std::thread;

    let function = ArcBiMutatingFunction::new(|x: &mut i32, y: &mut i32| {
        *x += 1;
        *y += 2;
        *x + *y
    });

    let func1 = function.clone();
    let func2 = function.clone();

    let handle1 = thread::spawn(move || {
        let mut a = 10;
        let mut b = 20;
        func1.apply(&mut a, &mut b)
    });

    let handle2 = thread::spawn(move || {
        let mut a = 30;
        let mut b = 40;
        func2.apply(&mut a, &mut b)
    });

    let result1 = handle1.join().expect("thread should not panic");
    let result2 = handle2.join().expect("thread should not panic");

    assert_eq!(result1, 33); // (10+1) + (20+2) = 11 + 22 = 33
    assert_eq!(result2, 73); // (30+1) + (40+2) = 31 + 42 = 73
}

#[test]
fn test_arc_bi_mutating_function_name_and_set_name() {
    let mut swap_sum = ArcBiMutatingFunction::new_with_name("arc_function", |x: &mut i32, y: &mut i32| {
        std::mem::swap(&mut *x, &mut *y);
        *x + *y
    });

    assert_eq!(swap_sum.name(), Some("arc_function"));
    swap_sum.set_name("modified_arc");
    assert_eq!(swap_sum.name(), Some("modified_arc"));
}

#[test]
fn test_arc_bi_mutating_function_constant() {
    let constant = ArcBiMutatingFunction::constant(123);
    let mut a = 1;
    let mut b = 2;
    assert_eq!(constant.apply(&mut a, &mut b), 123);

    let cloned = constant.clone();
    let mut c = 10;
    let mut d = 20;
    assert_eq!(cloned.apply(&mut c, &mut d), 123);
}

#[test]
fn test_arc_bi_mutating_function_debug_display() {
    let swap_sum = ArcBiMutatingFunction::new(|x: &mut i32, y: &mut i32| {
        std::mem::swap(&mut *x, &mut *y);
        *x + *y
    });

    let debug_str = format!("{:?}", swap_sum);
    assert!(debug_str.contains("ArcBiMutatingFunction"));

    let display_str = format!("{}", swap_sum);
    assert!(display_str.contains("ArcBiMutatingFunction"));
}

// ============================================================================
// Function Composition Tests - and_then
// ============================================================================

// ============================================================================
// Conditional Function Tests - when/or_else
// ============================================================================

#[test]
fn test_box_conditional_bi_mutating_function() {
    let swap_and_sum = BoxBiMutatingFunction::new(|x: &mut i32, y: &mut i32| {
        std::mem::swap(&mut *x, &mut *y);
        *x + *y
    });

    let multiply = BoxBiMutatingFunction::new(|x: &mut i32, y: &mut i32| {
        *x *= *y;
        *x
    });

    let conditional = swap_and_sum.when(|x: &i32, _y: &i32| *x > 0).or_else(multiply);

    // Test when condition is true
    let mut a = 5;
    let mut b = 3;
    assert_eq!(conditional.apply(&mut a, &mut b), 8); // swap_and_sum executed

    // Test when condition is false
    let mut c = -5;
    let mut d = 3;
    assert_eq!(conditional.apply(&mut c, &mut d), -15); // multiply executed
}

#[test]
fn test_rc_conditional_bi_mutating_function() {
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

    // Test when condition is true
    let mut a = 5;
    let mut b = 3;
    assert_eq!(conditional.apply(&mut a, &mut b), 8); // swap_and_sum executed

    // Test cloned conditional
    let mut c = 10;
    let mut d = 2;
    assert_eq!(cloned.apply(&mut c, &mut d), 12); // swap_and_sum executed
}

#[test]
fn test_arc_conditional_bi_mutating_function() {
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

    // Test when condition is true
    let mut a = 5;
    let mut b = 3;
    assert_eq!(conditional.apply(&mut a, &mut b), 8); // swap_and_sum executed

    // Test cloned conditional
    let mut c = 10;
    let mut d = 2;
    assert_eq!(cloned.apply(&mut c, &mut d), 12); // swap_and_sum executed
}
