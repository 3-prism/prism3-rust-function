// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_function::ArcPredicate;
use qubit_function::ArcStatefulTransformer;
use qubit_function::BoxPredicate;
use qubit_function::BoxStatefulTransformer;
use qubit_function::Predicate;
use qubit_function::RcPredicate;
use qubit_function::RcStatefulTransformer;
use qubit_function::StatefulTransformer;

// ============================================================================
// BoxStatefulTransformer Tests
// ============================================================================

/// Custom StatefulTransformer struct for testing default into_xxx() methods
#[derive(Clone)]
struct CustomStatefulTransformer {
    multiplier: i32,
}

impl StatefulTransformer<i32, i32> for CustomStatefulTransformer {
    fn apply(&mut self, input: i32) -> i32 {
        self.multiplier += 1;
        input * self.multiplier
    }
}

/// Custom thread-safe StatefulTransformer struct
#[derive(Clone)]
struct CustomSendStatefulTransformer {
    multiplier: i32,
}

impl StatefulTransformer<i32, i32> for CustomSendStatefulTransformer {
    fn apply(&mut self, input: i32) -> i32 {
        self.multiplier += 1;
        input * self.multiplier
    }
}

/// Test custom StatefulTransformer with string types
#[derive(Clone)]
struct StringLengthStatefulTransformer {
    total_length: usize,
}

impl StatefulTransformer<String, String> for StringLengthStatefulTransformer {
    fn apply(&mut self, input: String) -> String {
        self.total_length += input.len();
        format!("[{}] {}", self.total_length, input)
    }
}

/// Test custom StatefulTransformer with complex state
struct StatefulStatefulTransformer {
    count: i32,
    sum: i32,
    history: Vec<i32>,
}

impl StatefulTransformer<i32, (i32, i32, usize)> for StatefulStatefulTransformer {
    fn apply(&mut self, input: i32) -> (i32, i32, usize) {
        self.count += 1;
        self.sum += input;
        self.history.push(input);
        (self.count, self.sum, self.history.len())
    }
}

// ============================================================================
// into_fn Tests
// ============================================================================

// ============================================================================
// Closure to_xxx Non-Consuming Conversion Tests
// ============================================================================

// ============================================================================
// TransformerOnce Implementation Tests
// ============================================================================

#[test]
fn test_rc_conditional_mapper_clone() {
    let conditional = RcStatefulTransformer::new(|x: i32| x * 2).when(|x: &i32| *x > 0);

    // Clone the RcConditionalStatefulTransformer before calling or_else
    let conditional_clone = conditional.clone();

    let mut mapper1 = conditional.or_else(|x: i32| -x);
    let mut mapper2 = conditional_clone.or_else(|x: i32| x + 100);

    // Both cloned conditional mappers work correctly
    assert_eq!(mapper1.apply(5), 10); // Condition satisfied: 5 * 2
    assert_eq!(mapper1.apply(-5), 5); // Condition not satisfied: -(-5)
    assert_eq!(mapper2.apply(5), 10); // Condition satisfied: 5 * 2
    assert_eq!(mapper2.apply(-5), 95); // Condition not satisfied: -5 + 100
}

// ============================================================================
// Complex Composition Tests
// ============================================================================

#[test]
fn test_complex_pipeline() {
    let mut counter1 = 0;
    let step1 = BoxStatefulTransformer::new(move |x: i32| {
        counter1 += 1;
        format!("Step1[{}]: {}", counter1, x)
    });

    let mut counter2 = 0;
    let step2 = BoxStatefulTransformer::new(move |s: String| {
        counter2 += 1;
        format!("{} -> Step2[{}]", s, counter2)
    });

    let mut counter3 = 0;
    let step3 = BoxStatefulTransformer::new(move |s: String| {
        counter3 += 1;
        format!("{} -> Step3[{}]", s, counter3)
    });

    let mut pipeline = step1.and_then(step2).and_then(step3);

    assert_eq!(pipeline.apply(10), "Step1[1]: 10 -> Step2[1] -> Step3[1]");
    assert_eq!(pipeline.apply(20), "Step1[2]: 20 -> Step2[2] -> Step3[2]");
}

#[test]
fn test_nested_conditional() {
    let mut valid_count = 0;
    let mut invalid_count = 0;
    let mut error_count = 0;

    let mut mapper = BoxStatefulTransformer::new(move |x: i32| {
        valid_count += 1;
        format!("Valid[{}]: {}", valid_count, x * 2)
    })
    .when(|x: &i32| *x > 0)
    .or_else(move |x: i32| {
        let mut sub_mapper = BoxStatefulTransformer::new(move |x: i32| {
            invalid_count += 1;
            format!("Invalid[{}]: {}", invalid_count, x + 100)
        })
        .when(move |x: &i32| *x < 0)
        .or_else(move |x: i32| {
            error_count += 1;
            format!("Error[{}]: {}", error_count, x)
        });
        sub_mapper.apply(x)
    });

    assert_eq!(mapper.apply(5), "Valid[1]: 10");
    assert_eq!(mapper.apply(-5), "Invalid[1]: 95");
    assert_eq!(mapper.apply(0), "Error[1]: 0");
    assert_eq!(mapper.apply(10), "Valid[2]: 20");
}

// ============================================================================
// State Modification Tests
// ============================================================================

#[test]
fn test_stateful_counting() {
    let mut count = 0;
    let mut mapper = BoxStatefulTransformer::new(move |x: i32| {
        count += 1;
        (x, count)
    });

    assert_eq!(mapper.apply(100), (100, 1));
    assert_eq!(mapper.apply(200), (200, 2));
    assert_eq!(mapper.apply(300), (300, 3));
}

#[test]
fn test_stateful_accumulation() {
    let mut sum = 0;
    let mut mapper = BoxStatefulTransformer::new(move |x: i32| {
        sum += x;
        sum
    });

    assert_eq!(mapper.apply(10), 10);
    assert_eq!(mapper.apply(20), 30);
    assert_eq!(mapper.apply(30), 60);
}

// ============================================================================
// Type Conversion Tests
// ============================================================================

#[test]
fn test_different_types() {
    let mut counter = 0;
    let mut mapper = BoxStatefulTransformer::new(move |x: i32| {
        counter += 1;
        format!("Item #{}: {}", counter, x)
    });

    assert_eq!(mapper.apply(100), "Item #1: 100");
    assert_eq!(mapper.apply(200), "Item #2: 200");
}

#[test]
fn test_string_to_length() {
    let mut total_length = 0;
    let mut mapper = BoxStatefulTransformer::new(move |s: String| {
        total_length += s.len();
        total_length
    });

    assert_eq!(mapper.apply("hello".to_string()), 5);
    assert_eq!(mapper.apply("world".to_string()), 10);
    assert_eq!(mapper.apply("!".to_string()), 11);
}

// ============================================================================
// Predicate Integration Tests
// ============================================================================

#[test]
fn test_with_arc_predicate() {
    let predicate = ArcPredicate::new(|x: &i32| *x > 0);

    let mut mapper = ArcStatefulTransformer::new(|x: i32| x * 2)
        .when(predicate.clone())
        .or_else(|x: i32| -x);

    assert_eq!(mapper.apply(5), 10);
    assert_eq!(mapper.apply(-5), 5);

    // Predicate still usable
    assert!(predicate.test(&10));
    assert!(!predicate.test(&-10));
}

#[test]
fn test_with_rc_predicate() {
    let predicate = RcPredicate::new(|x: &i32| *x > 0);

    let mut mapper = RcStatefulTransformer::new(|x: i32| x * 2)
        .when(predicate.clone())
        .or_else(|x: i32| -x);

    assert_eq!(mapper.apply(5), 10);
    assert_eq!(mapper.apply(-5), 5);

    // Predicate still usable
    assert!(predicate.test(&10));
    assert!(!predicate.test(&-10));
}

// ============================================================================
// Custom StatefulTransformer Default Implementation Tests
// ============================================================================

/// Test BoxStatefulTransformer implements TransformerOnce trait
#[test]
fn test_box_mapper_apply() {
    let mut counter = 0;
    let mut mapper = BoxStatefulTransformer::new(move |x: i32| {
        counter += 1;
        x + counter
    });

    // BoxStatefulTransformer can be consumed as TransformerOnce
    assert_eq!(StatefulTransformer::apply(&mut mapper, 10), 11); // 10 + 1
}

// ============================================================================
// Conditional StatefulTransformer Display/Debug Tests
// ============================================================================
