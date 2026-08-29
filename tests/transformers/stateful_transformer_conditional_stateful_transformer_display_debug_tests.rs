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

#[cfg(test)]
mod conditional_stateful_transformer_display_debug_tests {
    use super::ArcStatefulTransformer;
    use super::BoxStatefulTransformer;
    use super::RcStatefulTransformer;

    #[test]
    fn test_box_conditional_stateful_transformer_display() {
        let mut counter = 0;
        let add = BoxStatefulTransformer::new(move |x: i32| {
            counter += 1;
            x + counter
        });
        let conditional = add.when(|x: &i32| *x > 0);
        let display_str = format!("{}", conditional);
        assert!(display_str.contains("BoxConditionalStatefulTransformer"));
    }

    #[test]
    fn test_box_conditional_stateful_transformer_display_no_name() {
        let mut counter = 0;
        let add = BoxStatefulTransformer::new(move |x: i32| {
            counter += 1;
            x + counter
        });
        let conditional = add.when(|x: &i32| *x > 0);
        let display_str = format!("{}", conditional);
        assert_eq!(
            display_str,
            "BoxConditionalStatefulTransformer(BoxStatefulTransformer, BoxPredicate(unnamed))"
        );
    }

    #[test]
    fn test_box_conditional_stateful_transformer_debug() {
        let mut counter = 0;
        let add = BoxStatefulTransformer::new(move |x: i32| {
            counter += 1;
            x + counter
        });
        let conditional = add.when(|x: &i32| *x > 0);
        let debug_str = format!("{:?}", conditional);
        assert!(debug_str.contains("BoxConditionalStatefulTransformer"));
    }

    #[test]
    fn test_rc_conditional_stateful_transformer_display() {
        let mut counter = 0;
        let add = RcStatefulTransformer::new(move |x: i32| {
            counter += 1;
            x + counter
        });
        let conditional = add.when(|x: &i32| *x > 0);
        let display_str = format!("{}", conditional);
        assert!(display_str.contains("RcConditionalStatefulTransformer"));
    }

    #[test]
    fn test_rc_conditional_stateful_transformer_display_no_name() {
        let mut counter = 0;
        let add = RcStatefulTransformer::new(move |x: i32| {
            counter += 1;
            x + counter
        });
        let conditional = add.when(|x: &i32| *x > 0);
        let display_str = format!("{}", conditional);
        assert_eq!(
            display_str,
            "RcConditionalStatefulTransformer(RcStatefulTransformer, RcPredicate(unnamed))"
        );
    }

    #[test]
    fn test_rc_conditional_stateful_transformer_debug() {
        let mut counter = 0;
        let add = RcStatefulTransformer::new(move |x: i32| {
            counter += 1;
            x + counter
        });
        let conditional = add.when(|x: &i32| *x > 0);
        let debug_str = format!("{:?}", conditional);
        assert!(debug_str.contains("RcConditionalStatefulTransformer"));
    }

    #[test]
    fn test_arc_conditional_stateful_transformer_display() {
        let mut counter = 0;
        let add = ArcStatefulTransformer::new(move |x: i32| {
            counter += 1;
            x + counter
        });
        let conditional = add.when(|x: &i32| *x > 0);
        let display_str = format!("{}", conditional);
        assert!(display_str.contains("ArcConditionalStatefulTransformer"));
    }

    #[test]
    fn test_arc_conditional_stateful_transformer_display_no_name() {
        let mut counter = 0;
        let add = ArcStatefulTransformer::new(move |x: i32| {
            counter += 1;
            x + counter
        });
        let conditional = add.when(|x: &i32| *x > 0);
        let display_str = format!("{}", conditional);
        assert_eq!(
            display_str,
            "ArcConditionalStatefulTransformer(ArcStatefulTransformer, ArcPredicate(unnamed))"
        );
    }

    #[test]
    fn test_arc_conditional_stateful_transformer_debug() {
        let mut counter = 0;
        let add = ArcStatefulTransformer::new(move |x: i32| {
            counter += 1;
            x + counter
        });
        let conditional = add.when(|x: &i32| *x > 0);
        let debug_str = format!("{:?}", conditional);
        assert!(debug_str.contains("ArcConditionalStatefulTransformer"));
    }
}

// ============================================================================
// Basic StatefulTransformer Display Tests
// ============================================================================
