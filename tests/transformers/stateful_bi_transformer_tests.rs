// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_function::ArcStatefulBiTransformer;
use qubit_function::ArcStatefulBinaryOperator;
use qubit_function::ArcStatefulTransformer;
use qubit_function::BoxBiPredicate;
use qubit_function::BoxStatefulBiTransformer;
use qubit_function::BoxStatefulBinaryOperator;
use qubit_function::BoxStatefulTransformer;
use qubit_function::RcStatefulBiTransformer;
use qubit_function::RcStatefulBinaryOperator;
use qubit_function::RcStatefulTransformer;
use qubit_function::StatefulBiTransformer;
use qubit_function::StatefulBinaryOperator;

#[test]
fn test_stateful_binary_operator_trait_bound() {
    fn reduce<T, O>(values: Vec<T>, initial: T, mut op: O) -> T
    where
        O: StatefulBinaryOperator<T>,
    {
        values.into_iter().fold(initial, |acc, value| op.apply(acc, value))
    }

    let sum = BoxStatefulBiTransformer::new(|a: i32, b: i32| a + b);
    assert_eq!(reduce(vec![1, 2, 3, 4], 0, sum), 10);
}

#[test]
fn test_stateful_binary_operator_aliases() {
    let mut box_add: BoxStatefulBinaryOperator<i32> = BoxStatefulBinaryOperator::new(|a, b| a + b);
    assert_eq!(box_add.apply(20, 22), 42);

    let mut arc_mul: ArcStatefulBinaryOperator<i32> = ArcStatefulBinaryOperator::new(|a, b| a * b);
    assert_eq!(arc_mul.apply(6, 7), 42);

    let mut rc_max: RcStatefulBinaryOperator<i32> = RcStatefulBinaryOperator::new(|a, b| if a > b { a } else { b });
    assert_eq!(rc_max.apply(30, 42), 42);
}

// ============================================================================
// BoxStatefulBiTransformer Tests
// ============================================================================

#[test]
fn test_box_stateful_bi_transformer_new() {
    // Test basic creation and usage with stateful transformation
    let mut counter = 0;
    let mut transformer = BoxStatefulBiTransformer::new(move |x: i32, y: i32| {
        counter += 1;
        x + y + counter
    });

    assert_eq!(transformer.apply(10, 20), 31); // 10 + 20 + 1
    assert_eq!(transformer.apply(10, 20), 32); // 10 + 20 + 2
    assert_eq!(transformer.apply(10, 20), 33); // 10 + 20 + 3
}

#[test]
fn test_box_stateful_bi_transformer_constant() {
    // Test constant bi-transformer that ignores inputs
    let mut constant = BoxStatefulBiTransformer::constant("hello");
    assert_eq!(constant.apply(1, 2), "hello");
    assert_eq!(constant.apply(3, 4), "hello");
    assert_eq!(constant.apply(5, 6), "hello");
}

#[test]
fn test_box_stateful_bi_transformer_and_then() {
    // Test composition with and_then method
    let mut counter1 = 0;
    let bi_trans = BoxStatefulBiTransformer::new(move |x: i32, y: i32| {
        counter1 += 1;
        x + y + counter1
    });

    let mut counter2 = 0;
    let trans = BoxStatefulTransformer::new(move |x: i32| {
        counter2 += 1;
        x * counter2
    });

    let mut composed = bi_trans.and_then(trans);
    assert_eq!(composed.apply(10, 20), 31); // (10 + 20 + 1) * 1
    assert_eq!(composed.apply(10, 20), 64); // (10 + 20 + 2) * 2
    assert_eq!(composed.apply(10, 20), 99); // (10 + 20 + 3) * 3
}

#[test]
fn test_box_stateful_bi_transformer_and_then_with_closure() {
    // Test and_then with a closure
    let mut counter = 0;
    let bi_trans = BoxStatefulBiTransformer::new(move |x: i32, y: i32| {
        counter += 1;
        x + y + counter
    });

    let mut composed = bi_trans.and_then(|x: i32| x * 2);
    assert_eq!(composed.apply(5, 5), 22); // (5 + 5 + 1) * 2
    assert_eq!(composed.apply(5, 5), 24); // (5 + 5 + 2) * 2
}

#[test]
fn test_box_stateful_bi_transformer_when_or_else() {
    // Test conditional execution with when and or_else
    let mut then_count = 0;
    let mut else_count = 0;

    let mut transformer = BoxStatefulBiTransformer::new(move |x: i32, y: i32| {
        then_count += 1;
        format!("Then[{}]: {}", then_count, x + y)
    })
    .when(|x: &i32, y: &i32| *x > 0 && *y > 0)
    .or_else(move |x, y| {
        else_count += 1;
        format!("Else[{}]: {}", else_count, x * y)
    });

    assert_eq!(transformer.apply(5, 3), "Then[1]: 8");
    assert_eq!(transformer.apply(-5, 3), "Else[1]: -15");
    assert_eq!(transformer.apply(10, 2), "Then[2]: 12");
    assert_eq!(transformer.apply(0, 5), "Else[2]: 0");
}

#[test]
fn test_box_stateful_bi_transformer_when_with_predicate() {
    // Test when with a predicate object
    let predicate = BoxBiPredicate::new(|x: &i32, y: &i32| *x >= 10 && *y >= 10);

    let mut transformer = BoxStatefulBiTransformer::new(|x: i32, y: i32| x + y)
        .when(predicate)
        .or_else(|x, y| x * y);

    assert_eq!(transformer.apply(15, 20), 35); // both >= 10, add
    assert_eq!(transformer.apply(5, 20), 100); // not both >= 10, multiply
}

#[test]
fn test_box_stateful_bi_transformer_with_string_types() {
    // Test with string input and output types
    let mut count = 0;
    let mut transformer = BoxStatefulBiTransformer::new(move |s1: String, s2: String| {
        count += 1;
        format!("[{}] {}{}", count, s1, s2)
    });

    assert_eq!(
        transformer.apply("hello".to_string(), "world".to_string()),
        "[1] helloworld"
    );
    assert_eq!(transformer.apply("foo".to_string(), "bar".to_string()), "[2] foobar");
}

#[test]
fn test_box_stateful_bi_transformer_different_types() {
    // Test with different input and output types
    let mut counter = 0;
    let mut transformer = BoxStatefulBiTransformer::new(move |name: String, age: i32| {
        counter += 1;
        format!("#{} {} is {}", counter, name, age)
    });

    assert_eq!(transformer.apply("Alice".to_string(), 30), "#1 Alice is 30");
    assert_eq!(transformer.apply("Bob".to_string(), 25), "#2 Bob is 25");
}

#[test]
fn test_box_stateful_bi_transformer_accumulation() {
    // Test stateful accumulation
    let mut sum = 0;
    let mut transformer = BoxStatefulBiTransformer::new(move |x: i32, y: i32| {
        sum += x + y;
        sum
    });

    assert_eq!(transformer.apply(10, 20), 30);
    assert_eq!(transformer.apply(5, 5), 40);
    assert_eq!(transformer.apply(3, 7), 50);
}

#[test]
fn test_box_stateful_bi_transformer_complex_state() {
    // Test with complex internal state
    let mut history = Vec::new();
    let mut transformer = BoxStatefulBiTransformer::new(move |x: i32, y: i32| {
        let sum = x + y;
        history.push(sum);
        (sum, history.len())
    });

    assert_eq!(transformer.apply(10, 20), (30, 1));
    assert_eq!(transformer.apply(5, 5), (10, 2));
    assert_eq!(transformer.apply(3, 7), (10, 3));
}

// ============================================================================
// ArcStatefulBiTransformer Tests
// ============================================================================

#[test]
fn test_arc_stateful_bi_transformer_new() {
    // Test basic creation and usage
    let mut counter = 0;
    let mut transformer = ArcStatefulBiTransformer::new(move |x: i32, y: i32| {
        counter += 1;
        x + y + counter
    });

    assert_eq!(transformer.apply(10, 20), 31);
    assert_eq!(transformer.apply(10, 20), 32);
    assert_eq!(transformer.apply(10, 20), 33);
}

#[test]
fn test_arc_stateful_bi_transformer_constant() {
    // Test constant bi-transformer
    let mut constant = ArcStatefulBiTransformer::constant("hello");
    assert_eq!(constant.apply(1, 2), "hello");
    assert_eq!(constant.apply(3, 4), "hello");
}

#[test]
fn test_arc_stateful_bi_transformer_clone() {
    // Test cloning and shared state
    let mut counter = 0;
    let transformer = ArcStatefulBiTransformer::new(move |x: i32, y: i32| {
        counter += 1;
        x + y + counter
    });

    let mut t1 = transformer.clone();
    let mut t2 = transformer.clone();

    assert_eq!(t1.apply(10, 20), 31); // counter = 1
    assert_eq!(t2.apply(10, 20), 32); // counter = 2 (shared state)
    assert_eq!(t1.apply(10, 20), 33); // counter = 3 (shared state)
}

#[test]
fn test_arc_stateful_bi_transformer_and_then() {
    // Test composition with and_then
    let mut counter1 = 0;
    let bi_trans = ArcStatefulBiTransformer::new(move |x: i32, y: i32| {
        counter1 += 1;
        x + y + counter1
    });

    let mut counter2 = 0;
    let trans = ArcStatefulTransformer::new(move |x: i32| {
        counter2 += 1;
        x * counter2
    });

    let mut composed = bi_trans.and_then(trans);
    assert_eq!(composed.apply(10, 20), 31); // (10 + 20 + 1) * 1
    assert_eq!(composed.apply(10, 20), 64); // (10 + 20 + 2) * 2
}

#[test]
fn test_arc_stateful_bi_transformer_and_then_preserves_original() {
    // Test that and_then uses &self and preserves original
    let mut counter = 0;
    let bi_trans = ArcStatefulBiTransformer::new(move |x: i32, y: i32| {
        counter += 1;
        x + y + counter
    });

    let trans = ArcStatefulTransformer::new(|x: i32| x * 2);
    let mut composed = bi_trans.and_then(trans);

    assert_eq!(composed.apply(5, 5), 22); // (5 + 5 + 1) * 2

    // Original bi_trans still usable
    let mut original = bi_trans.clone();
    assert_eq!(original.apply(10, 20), 32); // 10 + 20 + 2 (state continues)
}

#[test]
fn test_arc_stateful_bi_transformer_when_or_else() {
    // Test conditional execution
    let mut then_count = 0;
    let mut else_count = 0;

    let mut transformer = ArcStatefulBiTransformer::new(move |x: i32, y: i32| {
        then_count += 1;
        format!("Then[{}]: {}", then_count, x + y)
    })
    .when(|x: &i32, y: &i32| *x > 0 && *y > 0)
    .or_else(move |x, y| {
        else_count += 1;
        format!("Else[{}]: {}", else_count, x * y)
    });

    assert_eq!(transformer.apply(5, 3), "Then[1]: 8");
    assert_eq!(transformer.apply(-5, 3), "Else[1]: -15");
}

#[test]
fn test_arc_stateful_bi_transformer_when_preserves_original() {
    // Test that when uses &self and preserves original
    let transformer = ArcStatefulBiTransformer::new(|x: i32, y: i32| x + y);
    let conditional = transformer.when(|x: &i32, _y: &i32| *x > 0);

    let mut result = conditional.or_else(|x, y| x * y);
    assert_eq!(result.apply(5, 3), 8);

    // Original transformer still usable
    let mut original = transformer.clone();
    assert_eq!(original.apply(10, 20), 30);
}

#[test]
fn test_arc_stateful_bi_transformer_thread_safe() {
    // Test thread safety
    use std::thread;

    let mut counter = 0;
    let transformer = ArcStatefulBiTransformer::new(move |x: i32, y: i32| {
        counter += 1;
        x + y + counter
    });

    let t1 = transformer.clone();
    let t2 = transformer.clone();

    let handle1 = thread::spawn(move || {
        let mut t = t1.clone();
        t.apply(10, 20)
    });

    let handle2 = thread::spawn(move || {
        let mut t = t2.clone();
        t.apply(5, 5)
    });

    let result1 = handle1.join().expect("thread should not panic");
    let result2 = handle2.join().expect("thread should not panic");

    // Results depend on execution order, but both should be valid
    assert!((31..=32).contains(&result1));
    assert!((11..=12).contains(&result2));
}

// ============================================================================
// RcStatefulBiTransformer Tests
// ============================================================================

#[test]
fn test_rc_stateful_bi_transformer_new() {
    // Test basic creation and usage
    let mut counter = 0;
    let mut transformer = RcStatefulBiTransformer::new(move |x: i32, y: i32| {
        counter += 1;
        x + y + counter
    });

    assert_eq!(transformer.apply(10, 20), 31);
    assert_eq!(transformer.apply(10, 20), 32);
    assert_eq!(transformer.apply(10, 20), 33);
}

#[test]
fn test_rc_stateful_bi_transformer_constant() {
    // Test constant bi-transformer
    let mut constant = RcStatefulBiTransformer::constant("hello");
    assert_eq!(constant.apply(1, 2), "hello");
    assert_eq!(constant.apply(3, 4), "hello");
}

#[test]
fn test_rc_stateful_bi_transformer_clone() {
    // Test cloning and shared state
    let mut counter = 0;
    let transformer = RcStatefulBiTransformer::new(move |x: i32, y: i32| {
        counter += 1;
        x + y + counter
    });

    let mut t1 = transformer.clone();
    let mut t2 = transformer.clone();

    assert_eq!(t1.apply(10, 20), 31); // counter = 1
    assert_eq!(t2.apply(10, 20), 32); // counter = 2 (shared state)
    assert_eq!(t1.apply(10, 20), 33); // counter = 3 (shared state)
}

#[test]
fn test_rc_stateful_bi_transformer_and_then() {
    // Test composition with and_then
    let mut counter1 = 0;
    let bi_trans = RcStatefulBiTransformer::new(move |x: i32, y: i32| {
        counter1 += 1;
        x + y + counter1
    });

    let mut counter2 = 0;
    let trans = RcStatefulTransformer::new(move |x: i32| {
        counter2 += 1;
        x * counter2
    });

    let mut composed = bi_trans.and_then(trans);
    assert_eq!(composed.apply(10, 20), 31); // (10 + 20 + 1) * 1
    assert_eq!(composed.apply(10, 20), 64); // (10 + 20 + 2) * 2
}

#[test]
fn test_rc_stateful_bi_transformer_and_then_preserves_original() {
    // Test that and_then uses &self and preserves original
    let mut counter = 0;
    let bi_trans = RcStatefulBiTransformer::new(move |x: i32, y: i32| {
        counter += 1;
        x + y + counter
    });

    let trans = RcStatefulTransformer::new(|x: i32| x * 2);
    let mut composed = bi_trans.and_then(trans);

    assert_eq!(composed.apply(5, 5), 22); // (5 + 5 + 1) * 2

    // Original bi_trans still usable
    let mut original = bi_trans.clone();
    assert_eq!(original.apply(10, 20), 32); // 10 + 20 + 2 (state continues)
}

#[test]
fn test_rc_stateful_bi_transformer_when_or_else() {
    // Test conditional execution
    let mut then_count = 0;
    let mut else_count = 0;

    let mut transformer = RcStatefulBiTransformer::new(move |x: i32, y: i32| {
        then_count += 1;
        format!("Then[{}]: {}", then_count, x + y)
    })
    .when(|x: &i32, y: &i32| *x > 0 && *y > 0)
    .or_else(move |x, y| {
        else_count += 1;
        format!("Else[{}]: {}", else_count, x * y)
    });

    assert_eq!(transformer.apply(5, 3), "Then[1]: 8");
    assert_eq!(transformer.apply(-5, 3), "Else[1]: -15");
}
