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
fn test_box_mapper_new() {
    let mut counter = 0;
    let mut mapper = BoxStatefulTransformer::new(move |x: i32| {
        counter += 1;
        x + counter
    });

    assert_eq!(mapper.apply(10), 11);
    assert_eq!(mapper.apply(10), 12);
    assert_eq!(mapper.apply(10), 13);
}

#[test]
fn test_box_mapper_identity() {
    let mut identity = BoxStatefulTransformer::<i32, i32>::identity();
    assert_eq!(identity.apply(42), 42);
    assert_eq!(identity.apply(100), 100);
}

#[test]
fn test_box_mapper_constant() {
    let mut constant = BoxStatefulTransformer::constant("hello");
    assert_eq!(constant.apply(1), "hello");
    assert_eq!(constant.apply(2), "hello");
    assert_eq!(constant.apply(3), "hello");
}

#[test]
fn test_box_mapper_and_then_migrated() {
    let mut counter1 = 0;
    let mapper1 = BoxStatefulTransformer::new(move |x: i32| {
        counter1 += 1;
        x + counter1
    });

    let mut counter2 = 0;
    let mapper2 = BoxStatefulTransformer::new(move |x: i32| {
        counter2 += 1;
        x * counter2
    });

    let mut composed = mapper1.and_then(mapper2);
    assert_eq!(composed.apply(10), 11); // (10 + 1) * 1
    assert_eq!(composed.apply(10), 24); // (10 + 2) * 2
    assert_eq!(composed.apply(10), 39); // (10 + 3) * 3
}

#[test]
fn test_box_mapper_and_then_with_closure() {
    let mut counter = 0;
    let mapper = BoxStatefulTransformer::new(move |x: i32| {
        counter += 1;
        x * counter
    });

    let mut composed = mapper.and_then(|x: i32| x + 1);
    assert_eq!(composed.apply(10), 11); // 10 * 1 + 1
    assert_eq!(composed.apply(10), 21); // 10 * 2 + 1
    assert_eq!(composed.apply(10), 31); // 10 * 3 + 1
}

#[test]
fn test_box_mapper_when_or_else() {
    let mut high_count = 0;
    let mut low_count = 0;

    let mut mapper = BoxStatefulTransformer::new(move |x: i32| {
        high_count += 1;
        format!("High[{}]: {}", high_count, x * 2)
    })
    .when(|x: &i32| *x >= 10)
    .or_else(move |x| {
        low_count += 1;
        format!("Low[{}]: {}", low_count, x + 1)
    });

    assert_eq!(mapper.apply(15), "High[1]: 30");
    assert_eq!(mapper.apply(5), "Low[1]: 6");
    assert_eq!(mapper.apply(20), "High[2]: 40");
    assert_eq!(mapper.apply(3), "Low[2]: 4");
}

// ============================================================================
// ArcStatefulTransformer Tests
// ============================================================================

#[test]
fn test_arc_mapper_new() {
    let mut counter = 0;
    let mut mapper = ArcStatefulTransformer::new(move |x: i32| {
        counter += 1;
        x + counter
    });

    assert_eq!(mapper.apply(10), 11);
    assert_eq!(mapper.apply(10), 12);
    assert_eq!(mapper.apply(10), 13);
}

#[test]
fn test_arc_mapper_identity() {
    let mut identity = ArcStatefulTransformer::<i32, i32>::identity();
    assert_eq!(identity.apply(42), 42);
    assert_eq!(identity.apply(100), 100);
}

#[test]
fn test_arc_mapper_constant() {
    let mut constant = ArcStatefulTransformer::constant("hello");
    assert_eq!(constant.apply(1), "hello");
    assert_eq!(constant.apply(2), "hello");
    assert_eq!(constant.apply(3), "hello");
}

#[test]
fn test_arc_mapper_clone() {
    let mut counter = 0;
    let mapper = ArcStatefulTransformer::new(move |x: i32| {
        counter += 1;
        x + counter
    });

    let mut mapper1 = mapper.clone();
    let mut mapper2 = mapper.clone();

    assert_eq!(mapper1.apply(10), 11);
    assert_eq!(mapper2.apply(10), 12);
    assert_eq!(mapper1.apply(10), 13);
}

#[test]
fn test_arc_mapper_and_then() {
    let mut counter1 = 0;
    let mapper1 = ArcStatefulTransformer::new(move |x: i32| {
        counter1 += 1;
        x + counter1
    });

    let mut counter2 = 0;
    let mapper2 = ArcStatefulTransformer::new(move |x: i32| {
        counter2 += 1;
        x * counter2
    });

    let mut composed = mapper1.and_then(mapper2);
    assert_eq!(composed.apply(10), 11); // (10 + 1) * 1
    assert_eq!(composed.apply(10), 24); // (10 + 2) * 2
    assert_eq!(composed.apply(10), 39); // (10 + 3) * 3
}

#[test]
fn test_arc_mapper_and_then_with_closure() {
    let mut counter = 0;
    let mapper = ArcStatefulTransformer::new(move |x: i32| {
        counter += 1;
        x * counter
    });

    let mut composed = mapper.and_then(|x: i32| x + 1);
    assert_eq!(composed.apply(10), 11); // 10 * 1 + 1
    assert_eq!(composed.apply(10), 21); // 10 * 2 + 1
    assert_eq!(composed.apply(10), 31); // 10 * 3 + 1
}

#[test]
fn test_arc_mapper_when_or_else() {
    let mut high_count = 0;
    let mut low_count = 0;

    let mut mapper = ArcStatefulTransformer::new(move |x: i32| {
        high_count += 1;
        format!("High[{}]: {}", high_count, x * 2)
    })
    .when(|x: &i32| *x >= 10)
    .or_else(move |x| {
        low_count += 1;
        format!("Low[{}]: {}", low_count, x + 1)
    });

    assert_eq!(mapper.apply(15), "High[1]: 30");
    assert_eq!(mapper.apply(5), "Low[1]: 6");
    assert_eq!(mapper.apply(20), "High[2]: 40");
    assert_eq!(mapper.apply(3), "Low[2]: 4");
}

// ============================================================================
// RcStatefulTransformer Tests
// ============================================================================

#[test]
fn test_rc_mapper_new() {
    let mut counter = 0;
    let mut mapper = RcStatefulTransformer::new(move |x: i32| {
        counter += 1;
        x + counter
    });

    assert_eq!(mapper.apply(10), 11);
    assert_eq!(mapper.apply(10), 12);
    assert_eq!(mapper.apply(10), 13);
}

#[test]
fn test_rc_mapper_identity() {
    let mut identity = RcStatefulTransformer::<i32, i32>::identity();
    assert_eq!(identity.apply(42), 42);
    assert_eq!(identity.apply(100), 100);
}

#[test]
fn test_rc_mapper_constant() {
    let mut constant = RcStatefulTransformer::constant("hello");
    assert_eq!(constant.apply(1), "hello");
    assert_eq!(constant.apply(2), "hello");
    assert_eq!(constant.apply(3), "hello");
}

#[test]
fn test_rc_mapper_clone() {
    let mut counter = 0;
    let mapper = RcStatefulTransformer::new(move |x: i32| {
        counter += 1;
        x + counter
    });

    let mut mapper1 = mapper.clone();
    let mut mapper2 = mapper.clone();

    assert_eq!(mapper1.apply(10), 11);
    assert_eq!(mapper2.apply(10), 12);
    assert_eq!(mapper1.apply(10), 13);
}

#[test]
fn test_rc_mapper_and_then() {
    let mut counter1 = 0;
    let mapper1 = RcStatefulTransformer::new(move |x: i32| {
        counter1 += 1;
        x + counter1
    });

    let mut counter2 = 0;
    let mapper2 = RcStatefulTransformer::new(move |x: i32| {
        counter2 += 1;
        x * counter2
    });

    let mut composed = mapper1.and_then(mapper2);
    assert_eq!(composed.apply(10), 11); // (10 + 1) * 1
    assert_eq!(composed.apply(10), 24); // (10 + 2) * 2
    assert_eq!(composed.apply(10), 39); // (10 + 3) * 3
}

#[test]
fn test_rc_mapper_and_then_with_closure() {
    let mut counter = 0;
    let mapper = RcStatefulTransformer::new(move |x: i32| {
        counter += 1;
        x * counter
    });

    let mut composed = mapper.and_then(|x: i32| x + 1);
    assert_eq!(composed.apply(10), 11); // 10 * 1 + 1
    assert_eq!(composed.apply(10), 21); // 10 * 2 + 1
    assert_eq!(composed.apply(10), 31); // 10 * 3 + 1
}

#[test]
fn test_rc_mapper_when_or_else() {
    let mut high_count = 0;
    let mut low_count = 0;

    let mut mapper = RcStatefulTransformer::new(move |x: i32| {
        high_count += 1;
        format!("High[{}]: {}", high_count, x * 2)
    })
    .when(|x: &i32| *x >= 10)
    .or_else(move |x| {
        low_count += 1;
        format!("Low[{}]: {}", low_count, x + 1)
    });

    assert_eq!(mapper.apply(15), "High[1]: 30");
    assert_eq!(mapper.apply(5), "Low[1]: 6");
    assert_eq!(mapper.apply(20), "High[2]: 40");
    assert_eq!(mapper.apply(3), "Low[2]: 4");
}

// ============================================================================
// Closure StatefulTransformer Tests
// ============================================================================

#[test]
fn test_closure_as_mapper() {
    let mut counter = 0;
    let mut mapper = |x: i32| {
        counter += 1;
        x + counter
    };

    assert_eq!(mapper.apply(10), 11);
}

// ============================================================================
// BoxStatefulTransformer composition tests
// ============================================================================

#[test]
fn test_box_mapper_and_then() {
    let mut counter1 = 0;
    let mapper1 = move |x: i32| {
        counter1 += 1;
        x + counter1
    };

    let mut counter2 = 0;
    let mapper2 = move |x: i32| {
        counter2 += 1;
        x * counter2
    };

    let mut composed = BoxStatefulTransformer::new(mapper1).and_then(mapper2);
    assert_eq!(composed.apply(10), 11); // (10 + 1) * 1
    assert_eq!(composed.apply(10), 24); // (10 + 2) * 2
}

#[test]
fn test_box_mapper_when() {
    let mut mapper = BoxStatefulTransformer::new(|x: i32| x * 2)
        .when(|x: &i32| *x > 0)
        .or_else(|x: i32| -x);

    assert_eq!(mapper.apply(5), 10);
    assert_eq!(mapper.apply(-5), 5);
}

// ============================================================================
// Conditional StatefulTransformer Tests
// ============================================================================

#[test]
fn test_box_conditional_mapper_with_predicate() {
    let predicate = BoxPredicate::new(|x: &i32| *x >= 10);

    let mut mapper = BoxStatefulTransformer::new(|x: i32| x * 2)
        .when(predicate)
        .or_else(|x| x + 1);

    assert_eq!(mapper.apply(15), 30);
    assert_eq!(mapper.apply(5), 6);
}

#[test]
fn test_arc_conditional_mapper_clone() {
    let conditional = ArcStatefulTransformer::new(|x: i32| x * 2).when(|x: &i32| *x > 0);

    // Clone the ArcConditionalStatefulTransformer before calling or_else
    let conditional_clone = conditional.clone();

    let mut mapper1 = conditional.or_else(|x: i32| -x);
    let mut mapper2 = conditional_clone.or_else(|x: i32| x + 100);

    // Both cloned conditional mappers work correctly
    assert_eq!(mapper1.apply(5), 10); // Condition satisfied: 5 * 2
    assert_eq!(mapper1.apply(-5), 5); // Condition not satisfied: -(-5)
    assert_eq!(mapper2.apply(5), 10); // Condition satisfied: 5 * 2
    assert_eq!(mapper2.apply(-5), 95); // Condition not satisfied: -5 + 100
}
