// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Unit tests for StatefulMutator types

use qubit_function::ArcStatefulMutator;
use qubit_function::BoxStatefulMutator;
use qubit_function::MutatorOnce;
use qubit_function::RcStatefulMutator;
use qubit_function::StatefulMutator;

// ============================================================================
// BoxStatefulMutator Tests
// ============================================================================

#[cfg(test)]
mod test_box_mutator {
    use super::BoxStatefulMutator;
    use super::StatefulMutator;

    #[test]
    fn test_new() {
        let mut mutator = BoxStatefulMutator::new(|x: &mut i32| *x += 1);
        let mut value = 5;
        mutator.apply(&mut value);
        assert_eq!(value, 6);
    }

    #[test]
    fn test_with_different_types() {
        // String
        let mut string_mutator = BoxStatefulMutator::new(|s: &mut String| s.push('!'));
        let mut text = String::from("hello");
        string_mutator.apply(&mut text);
        assert_eq!(text, "hello!");

        // Vec
        let mut vec_mutator = BoxStatefulMutator::new(|v: &mut Vec<i32>| v.push(42));
        let mut numbers = vec![1, 2, 3];
        vec_mutator.apply(&mut numbers);
        assert_eq!(numbers, vec![1, 2, 3, 42]);

        // bool
        let mut bool_mutator = BoxStatefulMutator::new(|b: &mut bool| *b = !*b);
        let mut flag = true;
        bool_mutator.apply(&mut flag);
        assert!(!flag);
    }

    #[test]
    fn test_and_then() {
        let mut mutator = BoxStatefulMutator::new(|x: &mut i32| *x *= 2).and_then(|x: &mut i32| *x += 10);

        let mut value = 5;
        mutator.apply(&mut value);
        assert_eq!(value, 20); // (5 * 2) + 10
    }

    #[test]
    fn test_and_then_multiple_chains() {
        let mut mutator = BoxStatefulMutator::new(|x: &mut i32| *x += 1)
            .and_then(|x: &mut i32| *x *= 2)
            .and_then(|x: &mut i32| *x -= 5);

        let mut value = 10;
        mutator.apply(&mut value);
        assert_eq!(value, 17); // ((10 + 1) * 2) - 5
    }

    #[test]
    fn test_and_then_with_box_consumer() {
        let c1 = BoxStatefulMutator::new(|x: &mut i32| *x *= 2);
        let c2 = BoxStatefulMutator::new(|x: &mut i32| *x += 10);
        let mut combined = c1.and_then(c2);

        let mut value = 5;
        combined.apply(&mut value);
        assert_eq!(value, 20);
    }

    #[test]
    fn test_noop() {
        let mut noop = BoxStatefulMutator::<i32>::noop();
        let mut value = 42;
        noop.apply(&mut value);
        assert_eq!(value, 42);
    }

    #[test]
    fn test_noop_with_different_types() {
        // Test with String
        let mut noop = BoxStatefulMutator::<String>::noop();
        let mut text = String::from("hello");
        noop.apply(&mut text);
        assert_eq!(text, "hello");

        // Test with Vec
        let mut noop = BoxStatefulMutator::<Vec<i32>>::noop();
        let mut numbers = vec![1, 2, 3];
        noop.apply(&mut numbers);
        assert_eq!(numbers, vec![1, 2, 3]);
    }

    #[test]
    fn test_noop_chaining() {
        let mut chained = BoxStatefulMutator::<i32>::noop()
            .and_then(|x: &mut i32| *x *= 2)
            .and_then(BoxStatefulMutator::<i32>::noop());

        let mut value = 5;
        chained.apply(&mut value);
        assert_eq!(value, 10);
    }

    #[test]
    fn test_if_then_true() {
        let mut mutator = BoxStatefulMutator::new(|x: &mut i32| *x += 10).when(|x: &i32| *x > 0);

        let mut value = 5;
        mutator.apply(&mut value);
        assert_eq!(value, 15);
    }

    #[test]
    fn test_if_then_false() {
        let mut mutator = BoxStatefulMutator::new(|x: &mut i32| *x += 10).when(|x: &i32| *x > 0);

        let mut value = -5;
        mutator.apply(&mut value);
        assert_eq!(value, -5); // unchanged
    }

    #[test]
    fn test_if_then_else() {
        let mut mutator = BoxStatefulMutator::new(|x: &mut i32| *x *= 2)
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
    fn test_new_with_name() {
        let mut mutator = BoxStatefulMutator::new_with_name("box_stateful_test", |x: &mut i32| *x += 1);
        assert_eq!(mutator.name(), Some("box_stateful_test"));

        let mut value = 5;
        mutator.apply(&mut value);
        assert_eq!(value, 6);
    }

    #[test]
    fn test_new_with_optional_name_some() {
        let mut mutator =
            BoxStatefulMutator::new_with_optional_name(|x: &mut i32| *x += 1, Some("box_optional".to_string()));
        assert_eq!(mutator.name(), Some("box_optional"));

        let mut value = 5;
        mutator.apply(&mut value);
        assert_eq!(value, 6);
    }

    #[test]
    fn test_new_with_optional_name_none() {
        let mut mutator = BoxStatefulMutator::new_with_optional_name(|x: &mut i32| *x += 1, None);
        assert_eq!(mutator.name(), None);

        let mut value = 5;
        mutator.apply(&mut value);
        assert_eq!(value, 6);
    }

    #[test]
    fn test_name_and_set_name() {
        let mut mutator = BoxStatefulMutator::new(|x: &mut i32| *x += 1);
        assert_eq!(mutator.name(), None);

        mutator.set_name("box_set_name");
        assert_eq!(mutator.name(), Some("box_set_name"));

        let mut value = 5;
        mutator.apply(&mut value);
        assert_eq!(value, 6);
    }

    // Note: BoxStatefulMutator cannot be safely converted to ArcStatefulMutator
    // because the inner function may not be Send. This test has been
    // removed.
}

// ============================================================================
// ArcStatefulMutator Tests
// ============================================================================
