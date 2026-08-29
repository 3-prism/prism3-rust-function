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
fn test_box_stateful_transformer_display_with_name() {
    let mut counter = 0;
    let transformer = BoxStatefulTransformer::new_with_name("counter", move |x: i32| {
        counter += 1;
        x + counter
    });
    let display_str = format!("{}", transformer);
    assert_eq!(display_str, "BoxStatefulTransformer(counter)");
}

#[test]
fn test_box_stateful_transformer_display_without_name() {
    let mut counter = 0;
    let transformer = BoxStatefulTransformer::new(move |x: i32| {
        counter += 1;
        x + counter
    });
    let display_str = format!("{}", transformer);
    assert_eq!(display_str, "BoxStatefulTransformer");
}

#[test]
fn test_rc_stateful_transformer_display_with_name() {
    let mut counter = 0;
    let transformer = RcStatefulTransformer::new_with_name("counter", move |x: i32| {
        counter += 1;
        x + counter
    });
    let display_str = format!("{}", transformer);
    assert_eq!(display_str, "RcStatefulTransformer(counter)");
}

#[test]
fn test_rc_stateful_transformer_display_without_name() {
    let mut counter = 0;
    let transformer = RcStatefulTransformer::new(move |x: i32| {
        counter += 1;
        x + counter
    });
    let display_str = format!("{}", transformer);
    assert_eq!(display_str, "RcStatefulTransformer");
}

#[test]
fn test_arc_stateful_transformer_display_with_name() {
    let mut counter = 0;
    let transformer = ArcStatefulTransformer::new_with_name("counter", move |x: i32| {
        counter += 1;
        x + counter
    });
    let display_str = format!("{}", transformer);
    assert_eq!(display_str, "ArcStatefulTransformer(counter)");
}

#[test]
fn test_arc_stateful_transformer_display_without_name() {
    let mut counter = 0;
    let transformer = ArcStatefulTransformer::new(move |x: i32| {
        counter += 1;
        x + counter
    });
    let display_str = format!("{}", transformer);
    assert_eq!(display_str, "ArcStatefulTransformer");
}

// ============================================================================
// StatefulTransformer Trait Default Methods Tests - into_once, to_once
// ============================================================================
