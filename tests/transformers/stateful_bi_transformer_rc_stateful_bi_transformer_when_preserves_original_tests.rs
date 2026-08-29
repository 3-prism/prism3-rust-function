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
fn test_rc_stateful_bi_transformer_when_preserves_original() {
    // Test that when uses &self and preserves original
    let transformer = RcStatefulBiTransformer::new(|x: i32, y: i32| x + y);
    let conditional = transformer.when(|x: &i32, _y: &i32| *x > 0);

    let mut result = conditional.or_else(|x, y| x * y);
    assert_eq!(result.apply(5, 3), 8);

    // Original transformer still usable
    let mut original = transformer.clone();
    assert_eq!(original.apply(10, 20), 30);
}

// ============================================================================
// Closure StatefulBiTransformer Tests
// ============================================================================

#[test]
fn test_closure_as_stateful_bi_transformer() {
    // Test that closures implement StatefulBiTransformer
    let mut counter = 0;
    let mut transformer = |x: i32, y: i32| {
        counter += 1;
        x + y + counter
    };

    // Use StatefulBiTransformer::apply which takes &mut self
    assert_eq!(transformer.apply(10, 20), 31);
}

// ============================================================================
// Concrete wrapper composition tests
// ============================================================================

// ============================================================================
// BoxConditionalStatefulBiTransformer Tests
// ============================================================================

#[test]
fn test_box_conditional_or_else_basic() {
    // Test basic or_else functionality
    let add = BoxStatefulBiTransformer::new(|x: i32, y: i32| x + y);
    let multiply = BoxStatefulBiTransformer::new(|x: i32, y: i32| x * y);

    let mut conditional = add.when(|x: &i32, y: &i32| *x > 0 && *y > 0).or_else(multiply);

    assert_eq!(conditional.apply(5, 3), 8); // both positive, add
    assert_eq!(conditional.apply(-5, 3), -15); // not both positive, multiply
    assert_eq!(conditional.apply(0, 5), 0); // zero, multiply
}

#[test]
fn test_box_conditional_or_else_with_closure() {
    // Test or_else with closure
    let add = BoxStatefulBiTransformer::new(|x: i32, y: i32| x + y);

    let mut conditional = add.when(|x: &i32, _y: &i32| *x > 10).or_else(|x, y| x - y);

    assert_eq!(conditional.apply(15, 5), 20); // x > 10, add
    assert_eq!(conditional.apply(5, 3), 2); // x <= 10, subtract
}

#[test]
fn test_box_conditional_stateful_transformers() {
    // Test conditional with stateful transformers
    let mut then_count = 0;
    let mut else_count = 0;

    let then_trans = BoxStatefulBiTransformer::new(move |x: i32, y: i32| {
        then_count += 1;
        format!("Then[{}]: {}", then_count, x + y)
    });

    let else_trans = BoxStatefulBiTransformer::new(move |x: i32, y: i32| {
        else_count += 1;
        format!("Else[{}]: {}", else_count, x * y)
    });

    let mut conditional = then_trans.when(|x: &i32, _y: &i32| *x > 0).or_else(else_trans);

    assert_eq!(conditional.apply(5, 3), "Then[1]: 8");
    assert_eq!(conditional.apply(-5, 3), "Else[1]: -15");
    assert_eq!(conditional.apply(10, 2), "Then[2]: 12");
}

// ============================================================================
// ArcConditionalStatefulBiTransformer Tests
// ============================================================================

#[test]
fn test_arc_conditional_or_else_basic() {
    // Test basic or_else functionality
    let add = ArcStatefulBiTransformer::new(|x: i32, y: i32| x + y);
    let multiply = ArcStatefulBiTransformer::new(|x: i32, y: i32| x * y);

    let mut conditional = add.when(|x: &i32, y: &i32| *x > 0 && *y > 0).or_else(multiply);

    assert_eq!(conditional.apply(5, 3), 8);
    assert_eq!(conditional.apply(-5, 3), -15);
}

#[test]
fn test_arc_conditional_clone() {
    // Test cloning of conditional transformer
    let add = ArcStatefulBiTransformer::new(|x: i32, y: i32| x + y);
    let conditional = add.when(|x: &i32, _y: &i32| *x > 0);

    let conditional_clone = conditional.clone();

    let mut result1 = conditional.or_else(|x, y| x * y);
    let mut result2 = conditional_clone.or_else(|x, y| x * y);

    assert_eq!(result1.apply(5, 3), 8);
    assert_eq!(result2.apply(5, 3), 8);
    assert_eq!(result1.apply(-5, 3), -15);
    assert_eq!(result2.apply(-5, 3), -15);
}

#[test]
fn test_arc_conditional_stateful_transformers() {
    // Test conditional with stateful transformers
    let mut then_count = 0;
    let mut else_count = 0;

    let then_trans = ArcStatefulBiTransformer::new(move |x: i32, y: i32| {
        then_count += 1;
        format!("Then[{}]: {}", then_count, x + y)
    });

    let else_trans = ArcStatefulBiTransformer::new(move |x: i32, y: i32| {
        else_count += 1;
        format!("Else[{}]: {}", else_count, x * y)
    });

    let mut conditional = then_trans.when(|x: &i32, _y: &i32| *x > 0).or_else(else_trans);

    assert_eq!(conditional.apply(5, 3), "Then[1]: 8");
    assert_eq!(conditional.apply(-5, 3), "Else[1]: -15");
}

// ============================================================================
// RcConditionalStatefulBiTransformer Tests
// ============================================================================

#[test]
fn test_rc_conditional_or_else_basic() {
    // Test basic or_else functionality
    let add = RcStatefulBiTransformer::new(|x: i32, y: i32| x + y);
    let multiply = RcStatefulBiTransformer::new(|x: i32, y: i32| x * y);

    let mut conditional = add.when(|x: &i32, y: &i32| *x > 0 && *y > 0).or_else(multiply);

    assert_eq!(conditional.apply(5, 3), 8);
    assert_eq!(conditional.apply(-5, 3), -15);
}

#[test]
fn test_rc_conditional_clone() {
    // Test cloning of conditional transformer
    let add = RcStatefulBiTransformer::new(|x: i32, y: i32| x + y);
    let conditional = add.when(|x: &i32, _y: &i32| *x > 0);

    let conditional_clone = conditional.clone();

    let mut result1 = conditional.or_else(|x, y| x * y);
    let mut result2 = conditional_clone.or_else(|x, y| x * y);

    assert_eq!(result1.apply(5, 3), 8);
    assert_eq!(result2.apply(5, 3), 8);
    assert_eq!(result1.apply(-5, 3), -15);
    assert_eq!(result2.apply(-5, 3), -15);
}

#[test]
fn test_rc_conditional_stateful_transformers() {
    // Test conditional with stateful transformers
    let mut then_count = 0;
    let mut else_count = 0;

    let then_trans = RcStatefulBiTransformer::new(move |x: i32, y: i32| {
        then_count += 1;
        format!("Then[{}]: {}", then_count, x + y)
    });

    let else_trans = RcStatefulBiTransformer::new(move |x: i32, y: i32| {
        else_count += 1;
        format!("Else[{}]: {}", else_count, x * y)
    });

    let mut conditional = then_trans.when(|x: &i32, _y: &i32| *x > 0).or_else(else_trans);

    assert_eq!(conditional.apply(5, 3), "Then[1]: 8");
    assert_eq!(conditional.apply(-5, 3), "Else[1]: -15");
}

// ============================================================================
// BiTransformerOnce Implementation Tests
// ============================================================================

#[test]
fn test_box_stateful_bi_transformer_apply() {
    // Test apply consuming the transformer
    let mut counter = 0;
    let mut transformer = BoxStatefulBiTransformer::new(move |x: i32, y: i32| {
        counter += 1;
        x + y + counter
    });

    assert_eq!(transformer.apply(10, 20), 31);
    // transformer is now consumed
}

#[test]
fn test_arc_stateful_bi_transformer_apply() {
    // Test apply for ArcStatefulBiTransformer
    let mut counter = 0;
    let mut transformer = ArcStatefulBiTransformer::new(move |x: i32, y: i32| {
        counter += 1;
        x + y + counter
    });

    assert_eq!(transformer.apply(10, 20), 31);
}

// ============================================================================
// Conditional StatefulBiTransformer Display/Debug Tests
// ============================================================================
