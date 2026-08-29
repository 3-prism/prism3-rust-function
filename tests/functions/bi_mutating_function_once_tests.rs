// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Comprehensive tests for BiMutatingFunctionOnce trait and its implementations

use qubit_function::BiMutatingFunctionOnce;
use qubit_function::BoxBiMutatingFunctionOnce;

// ============================================================================
// Helper Functions and Data Structures
// ============================================================================

fn append_strings_once(x: &mut String, y: &mut String) -> usize {
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

fn modify_structs_once(a: &mut TestStruct, b: &mut TestStruct) -> i32 {
    a.modify(b)
}

// ============================================================================
// BiMutatingFunctionOnce Trait Tests - Core Functionality
// ============================================================================

#[test]
fn test_bi_mutating_function_once_trait_apply() {
    // Test that BiMutatingFunctionOnce trait's apply method works correctly
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
fn test_bi_mutating_function_once_trait_apply_with_complex_types() {
    let modify = |a: &mut TestStruct, b: &mut TestStruct| a.modify(b);

    let mut s1 = TestStruct::new(10);
    let mut s2 = TestStruct::new(5);
    let result = modify.apply(&mut s1, &mut s2);

    assert_eq!(result, 25); // (10+5) + (5*2) = 15 + 10 = 25
    assert_eq!(s1.value, 15);
    assert_eq!(s2.value, 10);
}

// ============================================================================
// BoxBiMutatingFunctionOnce Tests
// ============================================================================

#[test]
fn test_box_bi_mutating_function_once_new() {
    let swap_sum = BoxBiMutatingFunctionOnce::new(|x: &mut i32, y: &mut i32| {
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
fn test_box_bi_mutating_function_once_new_allows_non_static_t() {
    fn run<'a>(value: &'a str) -> usize {
        let func: BoxBiMutatingFunctionOnce<&'a str, i32, usize> =
            BoxBiMutatingFunctionOnce::new(|x: &mut &'a str, y: &mut i32| x.len() + (*y as usize));
        let mut first = value;
        let mut second = 3;
        func.apply(&mut first, &mut second)
    }

    let text = String::from("hello");
    assert_eq!(run(text.as_str()), 8);
}

#[test]
fn test_box_bi_mutating_function_once_new_allows_non_static_u() {
    fn run<'a>(value: &'a str) -> usize {
        let func: BoxBiMutatingFunctionOnce<i32, &'a str, usize> =
            BoxBiMutatingFunctionOnce::new(|x: &mut i32, y: &mut &'a str| (*x as usize) + y.len());
        let mut first = 3;
        let mut second = value;
        func.apply(&mut first, &mut second)
    }

    let text = String::from("world");
    assert_eq!(run(text.as_str()), 8);
}

#[test]
fn test_box_bi_mutating_function_once_new_allows_non_static_r() {
    fn run<'a>(value: &'a str) -> &'a str {
        let func: BoxBiMutatingFunctionOnce<&'a str, i32, &'a str> =
            BoxBiMutatingFunctionOnce::new(|x: &mut &'a str, _y: &mut i32| *x);
        let mut first = value;
        let mut second = 0;
        func.apply(&mut first, &mut second)
    }

    let text = String::from("qubit");
    assert_eq!(run(text.as_str()), "qubit");
}

#[test]
fn test_box_bi_mutating_function_once_new_with_name() {
    let swap_sum = BoxBiMutatingFunctionOnce::new_with_name("swap_and_sum_once", |x: &mut i32, y: &mut i32| {
        std::mem::swap(&mut *x, &mut *y);
        *x + *y
    });
    assert_eq!(swap_sum.name(), Some("swap_and_sum_once"));
    let mut a = 10;
    let mut b = 15;
    assert_eq!(swap_sum.apply(&mut a, &mut b), 25);
}

#[test]
fn test_box_bi_mutating_function_once_new_with_optional_name() {
    let swap_sum = BoxBiMutatingFunctionOnce::new_with_optional_name(
        |x: &mut i32, y: &mut i32| {
            std::mem::swap(&mut *x, &mut *y);
            *x + *y
        },
        Some("test_function_once".to_string()),
    );
    assert_eq!(swap_sum.name(), Some("test_function_once"));

    let no_name = BoxBiMutatingFunctionOnce::new_with_optional_name(|x: &mut i32, y: &mut i32| *x + *y, None);
    assert_eq!(no_name.name(), None);
}

#[test]
fn test_box_bi_mutating_function_once_name_and_set_name() {
    let mut swap_sum = BoxBiMutatingFunctionOnce::new(|x: &mut i32, y: &mut i32| {
        std::mem::swap(&mut *x, &mut *y);
        *x + *y
    });

    assert_eq!(swap_sum.name(), None);
    swap_sum.set_name("modified_name_once");
    assert_eq!(swap_sum.name(), Some("modified_name_once"));
    swap_sum.set_name("another_name_once");
    assert_eq!(swap_sum.name(), Some("another_name_once"));
}

#[test]
fn test_box_bi_mutating_function_once_constant() {
    let constant = BoxBiMutatingFunctionOnce::constant(42);
    let mut a = 1;
    let mut b = 2;
    assert_eq!(constant.apply(&mut a, &mut b), 42);
    assert_eq!(a, 1); // inputs unchanged
    assert_eq!(b, 2);
}

#[test]
fn test_box_bi_mutating_function_once_debug_display() {
    let swap_sum = BoxBiMutatingFunctionOnce::new(|x: &mut i32, y: &mut i32| {
        std::mem::swap(&mut *x, &mut *y);
        *x + *y
    });

    let debug_str = format!("{:?}", swap_sum);
    assert!(debug_str.contains("BoxBiMutatingFunctionOnce"));

    let display_str = format!("{}", swap_sum);
    assert!(display_str.contains("BoxBiMutatingFunctionOnce"));
}

#[test]
fn test_box_bi_mutating_function_once_with_strings() {
    let append = BoxBiMutatingFunctionOnce::new(append_strings_once);
    let mut s1 = "hello".to_string();
    let mut s2 = "world".to_string();

    let result = append.apply(&mut s1, &mut s2);
    assert_eq!(result, 14 + 13); // "hello_modified".len() + "world_changed".len()
    assert_eq!(s1, "hello_modified");
    assert_eq!(s2, "world_changed");
}

#[test]
fn test_box_bi_mutating_function_once_with_structs() {
    let modify = BoxBiMutatingFunctionOnce::new(modify_structs_once);
    let mut s1 = TestStruct::new(10);
    let mut s2 = TestStruct::new(5);

    let result = modify.apply(&mut s1, &mut s2);
    assert_eq!(result, 25); // (10+5) + (5*2) = 15 + 10 = 25
    assert_eq!(s1.value, 15);
    assert_eq!(s2.value, 10);
}

#[test]
fn test_box_bi_mutating_function_once_one_time_use() {
    // Test that BoxBiMutatingFunctionOnce can only be used once
    let counter = std::rc::Rc::new(std::cell::RefCell::new(0));
    let counter_clone = std::rc::Rc::clone(&counter);

    let increment = BoxBiMutatingFunctionOnce::new(move |x: &mut i32, y: &mut i32| {
        *counter_clone.borrow_mut() += 1;
        *x += 1;
        *y += 1;
        *x + *y
    });

    let mut a = 10;
    let mut b = 20;
    assert_eq!(increment.apply(&mut a, &mut b), 32); // 11 + 21 = 32
    assert_eq!(*counter.borrow(), 1);
    assert_eq!(a, 11);
    assert_eq!(b, 21);
}

// ============================================================================
// Function Composition Tests - and_then
// ============================================================================

// ============================================================================
// Conditional Function Tests - when/or_else
// ============================================================================

#[test]
fn test_box_conditional_bi_mutating_function_once() {
    let swap_and_sum = BoxBiMutatingFunctionOnce::new(|x: &mut i32, y: &mut i32| {
        std::mem::swap(&mut *x, &mut *y);
        *x + *y
    });

    let multiply = BoxBiMutatingFunctionOnce::new(|x: &mut i32, y: &mut i32| {
        *x *= *y;
        *x
    });

    let conditional = swap_and_sum.when(|x: &i32, _y: &i32| *x > 0).or_else(multiply);

    // Test when condition is true
    let mut a = 5;
    let mut b = 3;
    assert_eq!(conditional.apply(&mut a, &mut b), 8); // swap_and_sum executed

    // Test when condition is false - create a new conditional since
    // BiMutatingFunctionOnce consumes self
    let conditional2 = BoxBiMutatingFunctionOnce::new(|x: &mut i32, y: &mut i32| {
        std::mem::swap(&mut *x, &mut *y);
        *x + *y
    })
    .when(|x: &i32, _y: &i32| *x > 0)
    .or_else(BoxBiMutatingFunctionOnce::new(|x: &mut i32, y: &mut i32| {
        *x *= *y;
        *x
    }));
    let mut c = -5;
    let mut d = 3;
    assert_eq!(conditional2.apply(&mut c, &mut d), -15); // multiply executed
}

#[test]
fn test_conditional_bi_mutating_function_once_with_structs() {
    let modify = BoxBiMutatingFunctionOnce::new(modify_structs_once);
    let no_op = BoxBiMutatingFunctionOnce::new(|_a: &mut TestStruct, _b: &mut TestStruct| 0);

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

    // Test when condition is false - create new conditional since
    // BiMutatingFunctionOnce consumes self
    let conditional2 = BoxBiMutatingFunctionOnce::new(modify_structs_once)
        .when(|a: &TestStruct, b: &TestStruct| a.value > 0 && b.value > 0)
        .or_else(BoxBiMutatingFunctionOnce::new(
            |_a: &mut TestStruct, _b: &mut TestStruct| 0,
        ));
    let mut s3 = TestStruct::new(-10);
    let mut s4 = TestStruct::new(5);
    let result2 = conditional2.apply(&mut s3, &mut s4);
    assert_eq!(result2, 0); // no_op executed
    assert_eq!(s3.value, -10); // unchanged
    assert_eq!(s4.value, 5); // unchanged
}

// ============================================================================
// Edge Cases and Boundary Tests
// ============================================================================

#[test]
fn test_bi_mutating_function_once_with_zero_values() {
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
fn test_bi_mutating_function_once_with_negative_values() {
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
fn test_bi_mutating_function_once_with_large_values() {
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
fn test_bi_mutating_function_once_with_empty_strings() {
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
fn test_bi_mutating_function_once_with_unicode_strings() {
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
fn test_bi_mutating_function_once_identity_operations() {
    // Test functions that don't modify inputs
    let sum = |x: &mut i32, y: &mut i32| *x + *y;

    let mut a = 10;
    let mut b = 20;
    assert_eq!(sum.apply(&mut a, &mut b), 30);
    assert_eq!(a, 10); // unchanged
    assert_eq!(b, 20); // unchanged
}

#[test]
fn test_bi_mutating_function_once_chained_modifications() {
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
// Error and Panic Tests
// ============================================================================
