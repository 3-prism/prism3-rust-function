// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Comprehensive tests for BiFunctionOnce trait and BoxBiFunctionOnce

use qubit_function::ArcBiPredicate;
use qubit_function::BiFunctionOnce;
use qubit_function::BoxBiFunctionOnce;
use qubit_function::BoxBiPredicate;
use qubit_function::RcBiPredicate;

// ============================================================================
// BiFunctionOnce Trait Tests - Core Functionality
// ============================================================================

#[test]
fn test_bi_function_once_trait_apply() {
    // Test that BiFunctionOnce trait's apply method works correctly
    let add = |x: &i32, y: &i32| *x + *y;
    assert_eq!(add.apply(&21, &21), 42);
}

#[test]
fn test_bi_function_once_trait_apply_with_move() {
    // Test apply with moved value
    let value = String::from("hello");
    let concat = move |x: &String, y: &String| format!("{} {} {}", x, value, y);
    assert_eq!(
        concat.apply(&String::from("world"), &String::from("!")),
        "world hello !"
    );
}

#[test]
fn test_bi_function_once_trait_apply_with_different_types() {
    // Test apply with different input types
    let multiply = |x: &i32, y: &f64| *x as f64 * *y;
    assert_eq!(multiply.apply(&3, &2.5), 7.5);
}

// ============================================================================
// BoxBiFunctionOnce Tests - Box-based BiFunction Implementation
// ============================================================================

#[test]
fn test_box_bi_function_once_new() {
    // Test creating BoxBiFunctionOnce using new()
    let add = BoxBiFunctionOnce::new(|x: &i32, y: &i32| *x + *y);
    assert_eq!(add.apply(&5, &7), 12);
}

#[test]
fn test_box_bi_function_once_new_allows_non_static_t() {
    fn run<'a>(value: &'a str) -> usize {
        let func: BoxBiFunctionOnce<&'a str, i32, usize> =
            BoxBiFunctionOnce::new(|x: &&'a str, y: &i32| x.len() + (*y as usize));
        func.apply(&value, &3)
    }

    let text = String::from("hello");
    assert_eq!(run(text.as_str()), 8);
}

#[test]
fn test_box_bi_function_once_new_allows_non_static_u() {
    fn run<'a>(value: &'a str) -> usize {
        let func: BoxBiFunctionOnce<i32, &'a str, usize> =
            BoxBiFunctionOnce::new(|x: &i32, y: &&'a str| (*x as usize) + y.len());
        func.apply(&3, &value)
    }

    let text = String::from("world");
    assert_eq!(run(text.as_str()), 8);
}

#[test]
fn test_box_bi_function_once_new_allows_non_static_r() {
    fn run<'a>(value: &'a str) -> &'a str {
        let func: BoxBiFunctionOnce<&'a str, i32, &'a str> = BoxBiFunctionOnce::new(|x: &&'a str, _y: &i32| *x);
        func.apply(&value, &0)
    }

    let text = String::from("qubit");
    assert_eq!(run(text.as_str()), "qubit");
}

#[test]
fn test_box_bi_function_once_new_with_name() {
    // Test creating BoxBiFunctionOnce with name
    let add = BoxBiFunctionOnce::new_with_name("adder", |x: &i32, y: &i32| *x + *y);
    assert_eq!(add.name(), Some("adder"));
    assert_eq!(add.apply(&3, &4), 7);
}

#[test]
fn test_box_bi_function_once_new_with_optional_name() {
    // Test creating BoxBiFunctionOnce with optional name
    let add1 = BoxBiFunctionOnce::new_with_optional_name(|x: &i32, y: &i32| *x + *y, Some("named".to_string()));
    let add2 = BoxBiFunctionOnce::new_with_optional_name(|x: &i32, y: &i32| *x + *y, None);

    assert_eq!(add1.name(), Some("named"));
    assert_eq!(add2.name(), None);
    assert_eq!(add1.apply(&2, &3), 5);
    assert_eq!(add2.apply(&2, &3), 5);
}

#[test]
fn test_box_bi_function_once_name() {
    // Test getting name from BoxBiFunctionOnce
    let mut func = BoxBiFunctionOnce::new(|x: &i32, y: &i32| *x + *y);
    assert_eq!(func.name(), None);

    func.set_name("test");
    assert_eq!(func.name(), Some("test"));
}

#[test]
fn test_box_bi_function_once_set_name() {
    // Test setting name on BoxBiFunctionOnce
    let mut func = BoxBiFunctionOnce::new(|x: &i32, y: &i32| *x + *y);
    assert_eq!(func.name(), None);

    func.set_name("multiplier");
    assert_eq!(func.name(), Some("multiplier"));

    func.set_name("new name");
    assert_eq!(func.name(), Some("new name"));
}

#[test]
fn test_box_bi_function_once_constant() {
    // Test constant method on BoxBiFunctionOnce
    let constant_func1 = BoxBiFunctionOnce::constant(42);
    let constant_func2 = BoxBiFunctionOnce::constant(42);
    assert_eq!(constant_func1.apply(&1, &2), 42);
    assert_eq!(constant_func2.apply(&10, &20), 42);
}

#[test]
fn test_box_bi_function_once_apply() {
    // Test apply method on BoxBiFunctionOnce
    let multiply = BoxBiFunctionOnce::new(|x: &i32, y: &i32| *x * *y);
    assert_eq!(multiply.apply(&6, &7), 42);
}

// ============================================================================
// Concrete wrapper composition tests
// ============================================================================

// ============================================================================
// BoxConditionalBiFunctionOnce Tests - Conditional BiFunctions
// ============================================================================

#[test]
fn test_box_conditional_bi_function_once_when_or_else() {
    // Test when().or_else() method
    let add1 = BoxBiFunctionOnce::new(|x: &i32, y: &i32| *x + *y);
    let multiply1 = BoxBiFunctionOnce::new(|x: &i32, y: &i32| *x * *y);
    let conditional1 = add1.when(|x: &i32, y: &i32| *x > 0 && *y > 0).or_else(multiply1);

    let add2 = BoxBiFunctionOnce::new(|x: &i32, y: &i32| *x + *y);
    let multiply2 = BoxBiFunctionOnce::new(|x: &i32, y: &i32| *x * *y);
    let conditional2 = add2.when(|x: &i32, y: &i32| *x <= 0 || *y <= 0).or_else(multiply2);

    assert_eq!(conditional1.apply(&3, &4), 7); // when branch: 3 + 4 = 7
    assert_eq!(conditional2.apply(&-3, &4), 1); // when branch: -3 + 4 = 1
}

#[test]
fn test_box_conditional_bi_function_once_complex_conditions() {
    // Test with more complex conditions
    let add1 = BoxBiFunctionOnce::new(|x: &i32, y: &i32| *x + *y);
    let subtract1 = BoxBiFunctionOnce::new(|x: &i32, y: &i32| *x - *y);
    let conditional1 = add1.when(|x: &i32, y: &i32| *x >= *y).or_else(subtract1);

    let add2 = BoxBiFunctionOnce::new(|x: &i32, y: &i32| *x + *y);
    let subtract2 = BoxBiFunctionOnce::new(|x: &i32, y: &i32| *x - *y);
    let conditional2 = add2.when(|x: &i32, y: &i32| *x < *y).or_else(subtract2);

    assert_eq!(conditional1.apply(&5, &3), 8); // when branch: 5 >= 3, so 5 + 3 = 8
    assert_eq!(conditional2.apply(&3, &5), 8); // when branch: 3 < 5, so 3 + 5 = 8
}

#[test]
fn test_box_conditional_bi_function_once_with_string_operations() {
    // Test with string operations
    let concat1 = BoxBiFunctionOnce::new(|x: &String, y: &String| format!("{} {}", x, y));
    let reverse_concat1 = BoxBiFunctionOnce::new(|x: &String, y: &String| format!("{} {}", y, x));
    let conditional1 = concat1
        .when(|x: &String, y: &String| x.len() >= y.len())
        .or_else(reverse_concat1);

    let concat2 = BoxBiFunctionOnce::new(|x: &String, y: &String| format!("{} {}", x, y));
    let reverse_concat2 = BoxBiFunctionOnce::new(|x: &String, y: &String| format!("{} {}", y, x));
    let conditional2 = concat2
        .when(|x: &String, y: &String| x.len() < y.len())
        .or_else(reverse_concat2);

    assert_eq!(
        conditional1.apply(&String::from("hello"), &String::from("hi")),
        "hello hi"
    ); // when branch
    assert_eq!(
        conditional2.apply(&String::from("hi"), &String::from("hello")),
        "hi hello"
    ); // when branch
}

// ============================================================================
// Integration Tests - Complex Usage Patterns
// ============================================================================

#[test]
fn test_bi_function_once_with_custom_types() {
    // Test with custom types
    #[derive(Debug, PartialEq)]
    struct Point {
        x: i32,
        y: i32,
    }

    let add_points = |p1: &Point, p2: &Point| Point {
        x: p1.x + p2.x,
        y: p1.y + p2.y,
    };

    let p1 = Point { x: 1, y: 2 };
    let p2 = Point { x: 3, y: 4 };
    let result = add_points.apply(&p1, &p2);

    assert_eq!(result, Point { x: 4, y: 6 });
}

// ============================================================================
// Display and Debug Tests
// ============================================================================

#[test]
fn test_box_bi_function_once_display_without_name() {
    // Test Display implementation without name
    let func = BoxBiFunctionOnce::new(|x: &i32, y: &i32| *x + *y);
    let display = format!("{}", func);
    assert!(display.contains("BoxBiFunctionOnce"));
}

#[test]
fn test_box_bi_function_once_display_with_name() {
    // Test Display implementation with name
    let func = BoxBiFunctionOnce::new_with_name("adder", |x: &i32, y: &i32| *x + *y);
    let display = format!("{}", func);
    assert!(display.contains("adder"));
    assert!(display.contains("BoxBiFunctionOnce"));
}

#[test]
fn test_box_bi_function_once_debug() {
    // Test Debug implementation
    let func = BoxBiFunctionOnce::new(|x: &i32, y: &i32| *x + *y);
    let debug = format!("{:?}", func);
    assert!(debug.contains("BoxBiFunctionOnce"));
}

#[test]
fn test_box_conditional_bi_function_once_display() {
    // Test Display implementation for conditional
    let add = BoxBiFunctionOnce::new(|x: &i32, y: &i32| *x + *y);
    let conditional = add.when(|x: &i32, _y: &i32| *x > 0);
    let display = format!("{}", conditional);
    assert!(display.contains("BoxConditionalBiFunctionOnce"));
}

#[test]
fn test_box_conditional_bi_function_once_debug() {
    // Test Debug implementation for conditional
    let add = BoxBiFunctionOnce::new(|x: &i32, y: &i32| *x + *y);
    let conditional = add.when(|x: &i32, _y: &i32| *x > 0);
    let debug = format!("{:?}", conditional);
    assert!(debug.contains("BoxConditionalBiFunctionOnce"));
}

// ============================================================================
// Error Handling Tests
// ============================================================================

// ============================================================================
// Custom BiFunctionOnce Implementation Tests - Test Trait Default Methods
// ============================================================================

// ============================================================================
// Performance and Edge Cases
// ============================================================================

#[test]
fn test_bi_function_once_with_large_data() {
    // Test with large data structures
    let large_vec = vec![1; 10000];
    let combine = |v1: &Vec<i32>, v2: &Vec<i32>| {
        let mut result = v1.clone();
        result.extend(v2);
        result
    };

    let func = BoxBiFunctionOnce::new(combine);
    let result = func.apply(&large_vec, &large_vec);
    assert_eq!(result.len(), 20000);
}

#[test]
fn test_bi_function_once_consumption_semantics() {
    // Test that FnOnce semantics are properly maintained
    let counter = std::rc::Rc::new(std::cell::RefCell::new(0));

    let increment_counter = {
        let counter = counter.clone();
        move |x: &i32, y: &i32| {
            *counter.borrow_mut() += 1;
            *x + *y
        }
    };

    let func = BoxBiFunctionOnce::new(increment_counter);
    assert_eq!(*counter.borrow(), 0);

    // First call should work
    assert_eq!(func.apply(&1, &2), 3);
    assert_eq!(*counter.borrow(), 1);

    // Second call would consume the function (not applicable since it's moved)
}

#[test]
fn test_bi_function_once_with_references() {
    // Test with complex reference patterns
    let data = vec![1, 2, 3, 4, 5];
    let func = |slice: &[i32], index: &usize| slice.get(*index).copied().unwrap_or(0);

    assert_eq!(func(&data, &2), 3);
    assert_eq!(func(&data, &10), 0); // out of bounds
}

// ============================================================================
// Custom BiFunctionOnce Implementation Tests - Comprehensive Default Methods
// Testing
// ============================================================================
