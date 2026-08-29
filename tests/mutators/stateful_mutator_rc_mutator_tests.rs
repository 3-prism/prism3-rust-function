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
mod test_rc_mutator {
    use super::RcStatefulMutator;
    use super::StatefulMutator;

    #[test]
    fn test_new() {
        let mutator = RcStatefulMutator::new(|x: &mut i32| *x += 1);
        let mut value = 5;
        let mut c = mutator;
        c.apply(&mut value);
        assert_eq!(value, 6);
    }

    #[test]
    fn test_clone() {
        let mutator = RcStatefulMutator::new(|x: &mut i32| *x *= 2);
        let clone1 = mutator.clone();
        let clone2 = mutator.clone();

        let mut value1 = 5;
        let mut c1 = clone1;
        c1.apply(&mut value1);
        assert_eq!(value1, 10);

        let mut value2 = 3;
        let mut c2 = clone2;
        c2.apply(&mut value2);
        assert_eq!(value2, 6);
    }

    #[test]
    fn test_and_then() {
        let first = RcStatefulMutator::new(|x: &mut i32| *x *= 2);
        let second = RcStatefulMutator::new(|x: &mut i32| *x += 10);

        let chained = first.and_then(second.clone());

        let mut value = 5;
        let mut c = chained;
        c.apply(&mut value);
        assert_eq!(value, 20); // (5 * 2) + 10

        // first and second are still usable
        let mut value2 = 3;
        let mut f = first;
        f.apply(&mut value2);
        assert_eq!(value2, 6);
    }

    #[test]
    fn test_noop() {
        let noop = RcStatefulMutator::<i32>::noop();
        let mut value = 42;
        let mut m = noop;
        m.apply(&mut value);
        assert_eq!(value, 42);
    }

    #[test]
    fn test_noop_clone() {
        let noop = RcStatefulMutator::<i32>::noop();
        let clone1 = noop.clone();
        let clone2 = noop.clone();

        let mut value1 = 42;
        let mut m1 = clone1;
        m1.apply(&mut value1);
        assert_eq!(value1, 42);

        let mut value2 = 100;
        let mut m2 = clone2;
        m2.apply(&mut value2);
        assert_eq!(value2, 100);
    }

    #[test]
    fn test_noop_chaining() {
        let noop = RcStatefulMutator::<i32>::noop();
        let double = RcStatefulMutator::new(|x: &mut i32| *x *= 2);

        let chained = noop.and_then(double.clone());

        let mut value = 5;
        let mut c = chained;
        c.apply(&mut value);
        assert_eq!(value, 10);
    }

    // Note: RcStatefulMutator cannot be converted to ArcStatefulMutator because
    // Rc is not Send. This test has been removed.

    #[test]
    fn test_new_with_name() {
        let mut mutator = RcStatefulMutator::new_with_name("rc_stateful_test", |x: &mut i32| *x += 1);
        assert_eq!(mutator.name(), Some("rc_stateful_test"));

        let mut value = 5;
        mutator.apply(&mut value);
        assert_eq!(value, 6);
    }

    #[test]
    fn test_new_with_optional_name_some() {
        let mut mutator =
            RcStatefulMutator::new_with_optional_name(|x: &mut i32| *x += 1, Some("rc_stateful_optional".to_string()));
        assert_eq!(mutator.name(), Some("rc_stateful_optional"));

        let mut value = 5;
        mutator.apply(&mut value);
        assert_eq!(value, 6);
    }

    #[test]
    fn test_new_with_optional_name_none() {
        let mut mutator = RcStatefulMutator::new_with_optional_name(|x: &mut i32| *x += 1, None);
        assert_eq!(mutator.name(), None);

        let mut value = 5;
        mutator.apply(&mut value);
        assert_eq!(value, 6);
    }

    #[test]
    fn test_name_and_set_name() {
        let mut mutator = RcStatefulMutator::new(|x: &mut i32| *x += 1);
        assert_eq!(mutator.name(), None);

        mutator.set_name("rc_stateful_set_name");
        assert_eq!(mutator.name(), Some("rc_stateful_set_name"));

        let mut value = 5;
        mutator.apply(&mut value);
        assert_eq!(value, 6);
    }
}

// ============================================================================
// Closure Extension Methods Tests
// ============================================================================
