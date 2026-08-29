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
mod test_arc_mutator {
    use std::thread;

    use super::ArcMutator;
    use super::Mutator;

    #[test]
    fn test_new() {
        let mutator = ArcMutator::new(|x: &mut i32| *x += 1);
        let mut value = 5;
        mutator.apply(&mut value);
        assert_eq!(value, 6);
    }

    #[test]
    fn test_clone() {
        let mutator = ArcMutator::new(|x: &mut i32| *x *= 2);
        let clone1 = mutator.clone();
        let clone2 = mutator.clone();

        let mut value1 = 5;
        clone1.apply(&mut value1);
        assert_eq!(value1, 10);

        let mut value2 = 3;
        clone2.apply(&mut value2);
        assert_eq!(value2, 6);
    }

    #[test]
    fn test_and_then() {
        let first = ArcMutator::new(|x: &mut i32| *x *= 2);
        let second = ArcMutator::new(|x: &mut i32| *x += 10);

        let chained = first.and_then(second);

        let mut value = 5;
        chained.apply(&mut value);
        assert_eq!(value, 20); // (5 * 2) + 10

        // first and second are still usable
        let mut value2 = 3;
        first.apply(&mut value2);
        assert_eq!(value2, 6);
    }

    #[test]
    fn test_thread_safety() {
        let mutator = ArcMutator::new(|x: &mut i32| *x *= 2);
        let clone = mutator.clone();

        let handle = thread::spawn(move || {
            let mut value = 5;
            clone.apply(&mut value);
            value
        });

        let mut value = 3;
        mutator.apply(&mut value);
        assert_eq!(value, 6);

        assert_eq!(handle.join().expect("thread should not panic"), 10);
    }

    #[test]
    fn test_noop() {
        let noop = ArcMutator::<i32>::noop();
        let mut value = 42;
        noop.apply(&mut value);
        assert_eq!(value, 42);
    }

    #[test]
    fn test_noop_clone() {
        let noop = ArcMutator::<i32>::noop();
        let clone1 = noop.clone();
        let clone2 = noop.clone();

        let mut value1 = 42;
        clone1.apply(&mut value1);
        assert_eq!(value1, 42);

        let mut value2 = 100;
        clone2.apply(&mut value2);
        assert_eq!(value2, 100);
    }

    #[test]
    fn test_noop_chaining() {
        let noop = ArcMutator::<i32>::noop();
        let double = ArcMutator::new(|x: &mut i32| *x *= 2);

        let chained = noop.and_then(double);

        let mut value = 5;
        chained.apply(&mut value);
        assert_eq!(value, 10);
    }

    #[test]
    fn test_when() {
        let mutator = ArcMutator::new(|x: &mut i32| *x *= 2);
        let conditional = mutator.when(|x: &i32| *x > 0);

        let mut positive = 5;
        conditional.apply(&mut positive);
        assert_eq!(positive, 10);

        let mut negative = -5;
        conditional.apply(&mut negative);
        assert_eq!(negative, -5); // unchanged
    }

    #[test]
    fn test_conditional_or_else() {
        let mutator = ArcMutator::new(|x: &mut i32| *x *= 2)
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
    fn test_conditional_clone() {
        let conditional = ArcMutator::new(|x: &mut i32| *x *= 2).when(|x: &i32| *x > 0);
        let clone = conditional.clone();

        let mut positive = 5;
        conditional.apply(&mut positive);
        assert_eq!(positive, 10);

        let mut value2 = 3;
        clone.apply(&mut value2);
        assert_eq!(value2, 6);
    }

    #[test]
    fn test_new_with_name() {
        let mutator = ArcMutator::new_with_name("arc_test_mutator", |x: &mut i32| *x += 1);
        assert_eq!(mutator.name(), Some("arc_test_mutator"));

        let mut value = 5;
        mutator.apply(&mut value);
        assert_eq!(value, 6);
    }

    #[test]
    fn test_new_with_optional_name_some() {
        let mutator = ArcMutator::new_with_optional_name(|x: &mut i32| *x += 1, Some("arc_optional".to_string()));
        assert_eq!(mutator.name(), Some("arc_optional"));

        let mut value = 5;
        mutator.apply(&mut value);
        assert_eq!(value, 6);
    }

    #[test]
    fn test_new_with_optional_name_none() {
        let mutator = ArcMutator::new_with_optional_name(|x: &mut i32| *x += 1, None);
        assert_eq!(mutator.name(), None);

        let mut value = 5;
        mutator.apply(&mut value);
        assert_eq!(value, 6);
    }

    #[test]
    fn test_name_and_set_name() {
        let mut mutator = ArcMutator::new(|x: &mut i32| *x += 1);
        assert_eq!(mutator.name(), None);

        mutator.set_name("arc_set_name");
        assert_eq!(mutator.name(), Some("arc_set_name"));

        let mut value = 5;
        mutator.apply(&mut value);
        assert_eq!(value, 6);
    }
}

// ============================================================================
// RcMutator Tests
// ============================================================================
