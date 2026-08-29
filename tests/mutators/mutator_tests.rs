// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Unit tests for Mutator types (stateless Fn(&mut T))

use qubit_function::ArcMutator;
use qubit_function::BoxMutator;
use qubit_function::Mutator;
use qubit_function::MutatorOnce;
use qubit_function::RcMutator;

// ============================================================================
// Mutator Default Implementation Tests
// ============================================================================

/// Test struct that implements Mutator to test default methods
struct TestMutator {
    multiplier: i32,
}

impl TestMutator {
    fn new(multiplier: i32) -> Self {
        TestMutator { multiplier }
    }
}

impl Mutator<i32> for TestMutator {
    fn apply(&self, input: &mut i32) {
        *input *= self.multiplier;
    }
}

impl Clone for TestMutator {
    fn clone(&self) -> Self {
        TestMutator {
            multiplier: self.multiplier,
        }
    }
}

// ============================================================================
// BoxMutator Tests
// ============================================================================

#[cfg(test)]
mod test_box_mutator {
    use super::BoxMutator;
    use super::Mutator;

    #[test]
    fn test_new() {
        let mutator = BoxMutator::new(|x: &mut i32| *x += 1);
        let mut value = 5;
        mutator.apply(&mut value);
        assert_eq!(value, 6);
    }

    #[test]
    fn test_with_different_types() {
        // String
        let string_mutator = BoxMutator::new(|s: &mut String| s.push('!'));
        let mut text = String::from("hello");
        string_mutator.apply(&mut text);
        assert_eq!(text, "hello!");

        // Vec
        let vec_mutator = BoxMutator::new(|v: &mut Vec<i32>| v.push(42));
        let mut numbers = vec![1, 2, 3];
        vec_mutator.apply(&mut numbers);
        assert_eq!(numbers, vec![1, 2, 3, 42]);

        // bool
        let bool_mutator = BoxMutator::new(|b: &mut bool| *b = !*b);
        let mut flag = true;
        bool_mutator.apply(&mut flag);
        assert!(!flag);
    }

    #[test]
    fn test_and_then() {
        let mutator = BoxMutator::new(|x: &mut i32| *x *= 2).and_then(|x: &mut i32| *x += 10);

        let mut value = 5;
        mutator.apply(&mut value);
        assert_eq!(value, 20); // (5 * 2) + 10
    }

    #[test]
    fn test_and_then_multiple_chains() {
        let mutator = BoxMutator::new(|x: &mut i32| *x += 1)
            .and_then(|x: &mut i32| *x *= 2)
            .and_then(|x: &mut i32| *x -= 5);

        let mut value = 10;
        mutator.apply(&mut value);
        assert_eq!(value, 17); // ((10 + 1) * 2) - 5
    }

    #[test]
    fn test_and_then_with_box_mutator() {
        let c1 = BoxMutator::new(|x: &mut i32| *x *= 2);
        let c2 = BoxMutator::new(|x: &mut i32| *x += 10);
        let combined = c1.and_then(c2);

        let mut value = 5;
        combined.apply(&mut value);
        assert_eq!(value, 20);
    }

    #[test]
    fn test_noop() {
        let noop = BoxMutator::<i32>::noop();
        let mut value = 42;
        noop.apply(&mut value);
        assert_eq!(value, 42);
    }

    #[test]
    fn test_noop_with_different_types() {
        // Test with String
        let noop = BoxMutator::<String>::noop();
        let mut text = String::from("hello");
        noop.apply(&mut text);
        assert_eq!(text, "hello");

        // Test with Vec
        let noop = BoxMutator::<Vec<i32>>::noop();
        let mut numbers = vec![1, 2, 3];
        noop.apply(&mut numbers);
        assert_eq!(numbers, vec![1, 2, 3]);
    }

    #[test]
    fn test_noop_chaining() {
        let chained = BoxMutator::<i32>::noop()
            .and_then(|x: &mut i32| *x *= 2)
            .and_then(BoxMutator::<i32>::noop());

        let mut value = 5;
        chained.apply(&mut value);
        assert_eq!(value, 10);
    }

    #[test]
    fn test_if_then_true() {
        let mutator = BoxMutator::new(|x: &mut i32| *x += 10).when(|x: &i32| *x > 0);

        let mut value = 5;
        mutator.apply(&mut value);
        assert_eq!(value, 15);
    }

    #[test]
    fn test_if_then_false() {
        let mutator = BoxMutator::new(|x: &mut i32| *x += 10).when(|x: &i32| *x > 0);

        let mut value = -5;
        mutator.apply(&mut value);
        assert_eq!(value, -5); // unchanged
    }

    #[test]
    fn test_if_then_else() {
        let mutator = BoxMutator::new(|x: &mut i32| *x *= 2)
            .when(|x: &i32| *x > 0)
            .or_else(|x: &mut i32| *x = -*x);

        let mut positive = 10;
        mutator.apply(&mut positive);
        assert_eq!(positive, 20);

        let mut negative = -10;
        mutator.apply(&mut negative);
        assert_eq!(negative, 10);
    }

    #[test]
    fn test_conditional_and_then() {
        let cond1 = BoxMutator::new(|x: &mut i32| *x *= 2).when(|x: &i32| *x > 0);
        let cond2 = BoxMutator::new(|x: &mut i32| *x += 5);
        let chained = cond1.and_then(cond2);

        let mut positive = 10;
        chained.apply(&mut positive);
        assert_eq!(positive, 25); // (10 * 2) + 5

        let mut negative = -10;
        chained.apply(&mut negative);
        assert_eq!(negative, -5); // -10 + 5 (condition not met, only second mutator runs)
    }

    #[test]
    fn test_new_with_name() {
        let mutator = BoxMutator::new_with_name("test_mutator", |x: &mut i32| *x += 1);
        assert_eq!(mutator.name(), Some("test_mutator"));

        let mut value = 5;
        mutator.apply(&mut value);
        assert_eq!(value, 6);
    }

    #[test]
    fn test_new_with_optional_name_some() {
        let mutator = BoxMutator::new_with_optional_name(|x: &mut i32| *x += 1, Some("optional_name".to_string()));
        assert_eq!(mutator.name(), Some("optional_name"));

        let mut value = 5;
        mutator.apply(&mut value);
        assert_eq!(value, 6);
    }

    #[test]
    fn test_new_with_optional_name_none() {
        let mutator = BoxMutator::new_with_optional_name(|x: &mut i32| *x += 1, None);
        assert_eq!(mutator.name(), None);

        let mut value = 5;
        mutator.apply(&mut value);
        assert_eq!(value, 6);
    }

    #[test]
    fn test_name_and_set_name() {
        let mut mutator = BoxMutator::new(|x: &mut i32| *x += 1);
        assert_eq!(mutator.name(), None);

        mutator.set_name("set_name_test");
        assert_eq!(mutator.name(), Some("set_name_test"));

        let mut value = 5;
        mutator.apply(&mut value);
        assert_eq!(value, 6);
    }

    #[test]
    fn test_box_mutator_debug() {
        let mutator = BoxMutator::new(|x: &mut i32| *x *= 2);
        let debug_str = format!("{:?}", mutator);
        assert!(debug_str.contains("BoxMutator"));
        assert!(debug_str.contains("name"));
        assert!(debug_str.contains("function"));
    }

    #[test]
    fn test_box_mutator_display() {
        let mutator = BoxMutator::new(|x: &mut i32| *x *= 2);
        let display_str = format!("{}", mutator);
        assert_eq!(display_str, "BoxMutator");

        let named_mutator = BoxMutator::new_with_name("test_mutator", |x: &mut i32| *x *= 2);
        let named_display_str = format!("{}", named_mutator);
        assert_eq!(named_display_str, "BoxMutator(test_mutator)");
    }

    // Note: BoxMutator cannot be safely converted to ArcMutator because the
    // inner function may not be Send. This test has been removed.
}

// ============================================================================
// ArcMutator Tests
// ============================================================================
