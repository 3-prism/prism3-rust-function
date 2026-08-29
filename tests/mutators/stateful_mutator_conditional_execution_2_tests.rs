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
mod test_conditional_execution {
    use qubit_function::predicates::ArcPredicate;
    use qubit_function::predicates::BoxPredicate;
    use qubit_function::predicates::RcPredicate;

    use super::ArcStatefulMutator;
    use super::BoxStatefulMutator;
    use super::RcStatefulMutator;
    use super::StatefulMutator;

    // Helper function pointer for testing
    fn is_positive(x: &i32) -> bool {
        *x > 0
    }

    fn negate(x: &mut i32) {
        *x = -*x;
    }

    // ========================================================================
    // BoxStatefulMutator::when() tests
    // ========================================================================

    #[test]
    fn test_rc_conditional_clone() {
        let conditional = RcStatefulMutator::new(|x: &mut i32| *x *= 2).when(|x: &i32| *x > 0);
        let mut clone1 = conditional.clone();
        let mut clone2 = conditional.clone();

        let mut value1 = 5;
        clone1.apply(&mut value1);
        assert_eq!(value1, 10);

        let mut value2 = -5;
        clone2.apply(&mut value2);
        assert_eq!(value2, -5);
    }

    // ========================================================================
    // ArcStatefulMutator::when() tests
    // ========================================================================

    #[test]
    fn test_arc_when_with_closure() {
        let conditional = ArcStatefulMutator::new(|x: &mut i32| *x *= 2).when(|x: &i32| *x > 0);
        let mut m = conditional.clone();

        let mut positive = 5;
        m.apply(&mut positive);
        assert_eq!(positive, 10);

        let mut negative = -5;
        m.apply(&mut negative);
        assert_eq!(negative, -5);
    }

    #[test]
    fn test_arc_when_with_function_pointer() {
        let conditional = ArcStatefulMutator::new(|x: &mut i32| *x *= 2).when(is_positive as fn(&i32) -> bool);
        let mut m = conditional.clone();

        let mut positive = 5;
        m.apply(&mut positive);
        assert_eq!(positive, 10);

        let mut negative = -5;
        m.apply(&mut negative);
        assert_eq!(negative, -5);
    }

    #[test]
    fn test_arc_when_with_arc_predicate() {
        let pred = ArcPredicate::new(|x: &i32| *x > 0);
        let conditional = ArcStatefulMutator::new(|x: &mut i32| *x *= 2).when(pred);
        let mut m = conditional.clone();

        let mut positive = 5;
        m.apply(&mut positive);
        assert_eq!(positive, 10);

        let mut negative = -5;
        m.apply(&mut negative);
        assert_eq!(negative, -5);
    }

    // ========================================================================
    // ArcConditionalStatefulMutator::or_else() tests
    // ========================================================================

    #[test]
    fn test_arc_or_else_with_closure() {
        let mut mutator = ArcStatefulMutator::new(|x: &mut i32| *x *= 2)
            .when(|x: &i32| *x > 0)
            .or_else(|x: &mut i32| *x -= 1);

        let mut positive = 5;
        mutator.apply(&mut positive);
        assert_eq!(positive, 10);

        let mut negative = -5;
        mutator.apply(&mut negative);
        assert_eq!(negative, -6);
    }

    #[test]
    fn test_arc_or_else_with_function_pointer() {
        let mut mutator = ArcStatefulMutator::new(|x: &mut i32| *x *= 2)
            .when(|x: &i32| *x > 0)
            .or_else(negate as fn(&mut i32));

        let mut positive = 5;
        mutator.apply(&mut positive);
        assert_eq!(positive, 10);

        let mut negative = -5;
        mutator.apply(&mut negative);
        assert_eq!(negative, 5);
    }

    #[test]
    fn test_arc_or_else_with_arc_mutator() {
        let else_mutator = ArcStatefulMutator::new(|x: &mut i32| *x = 100);
        let mut mutator = ArcStatefulMutator::new(|x: &mut i32| *x *= 2)
            .when(|x: &i32| *x > 0)
            .or_else(else_mutator);

        let mut positive = 5;
        mutator.apply(&mut positive);
        assert_eq!(positive, 10);

        let mut negative = -5;
        mutator.apply(&mut negative);
        assert_eq!(negative, 100);
    }

    // Note: BoxStatefulMutator is not Send, so it cannot be used with
    // ArcStatefulMutator::or_else()

    // ========================================================================
    // ArcConditionalStatefulMutator::clone() tests
    // ========================================================================

    #[test]
    fn test_arc_conditional_clone() {
        let conditional = ArcStatefulMutator::new(|x: &mut i32| *x *= 2).when(|x: &i32| *x > 0);
        let mut clone1 = conditional.clone();
        let mut clone2 = conditional.clone();

        let mut value1 = 5;
        clone1.apply(&mut value1);
        assert_eq!(value1, 10);

        let mut value2 = -5;
        clone2.apply(&mut value2);
        assert_eq!(value2, -5);
    }

    // ========================================================================
    // Thread safety tests for ArcConditionalStatefulMutator
    // ========================================================================

    #[test]
    fn test_arc_conditional_thread_safety() {
        use std::thread;

        let conditional = ArcStatefulMutator::new(|x: &mut i32| *x *= 2).when(|x: &i32| *x > 0);
        let clone = conditional.clone();

        let handle = thread::spawn(move || {
            let mut value = 5;
            let mut m = clone;
            m.apply(&mut value);
            value
        });

        let mut value = -5;
        let mut m = conditional;
        m.apply(&mut value);
        assert_eq!(value, -5);

        assert_eq!(handle.join().expect("thread should not panic"), 10);
    }

    #[test]
    fn test_arc_or_else_thread_safety() {
        use std::thread;

        let mutator = ArcStatefulMutator::new(|x: &mut i32| *x *= 2)
            .when(|x: &i32| *x > 0)
            .or_else(|x: &mut i32| *x = 0);

        let clone = mutator.clone();

        let handle = thread::spawn(move || {
            let mut value = -5;
            let mut m = clone;
            m.apply(&mut value);
            value
        });

        let mut value = 5;
        let mut m = mutator;
        m.apply(&mut value);
        assert_eq!(value, 10);

        assert_eq!(handle.join().expect("thread should not panic"), 0);
    }

    // ========================================================================
    // Type conversion tests for ConditionalStatefulMutator
    // ========================================================================

    // ========================================================================
    // into_fn tests for ConditionalStatefulMutator
    // ========================================================================

    // ========================================================================
    // to_xxx tests for RcConditionalStatefulMutator
    // ========================================================================

    // ========================================================================
    // to_xxx tests for ArcConditionalStatefulMutator
    // ========================================================================

    // ========================================================================
    // Complex conditional composition tests
    // ========================================================================

    #[test]
    fn test_nested_conditionals() {
        // When x > 0: multiply by 2, then if result > 10: cap at 10
        let mut mutator = BoxStatefulMutator::new(|x: &mut i32| *x *= 2)
            .when(|x: &i32| *x > 0)
            .and_then(BoxStatefulMutator::new(|x: &mut i32| *x = 10).when(|x: &i32| *x > 10));

        let mut small = 3;
        mutator.apply(&mut small);
        assert_eq!(small, 6); // 3 * 2 = 6 (not capped)

        let mut medium = 5;
        mutator.apply(&mut medium);
        assert_eq!(medium, 10); // 5 * 2 = 10 (not capped)

        let mut large = 8;
        mutator.apply(&mut large);
        assert_eq!(large, 10); // 8 * 2 = 16 -> capped to 10

        let mut negative = -5;
        mutator.apply(&mut negative);
        assert_eq!(negative, -5); // Not doubled (condition failed)
    }

    #[test]
    fn test_or_else_chaining() {
        // If positive: double, else: triple
        let mut mutator = BoxStatefulMutator::new(|x: &mut i32| *x *= 2)
            .when(|x: &i32| *x > 0)
            .or_else(|x: &mut i32| *x *= 3);

        let mut positive = 5;
        mutator.apply(&mut positive);
        assert_eq!(positive, 10);

        let mut negative = -5;
        mutator.apply(&mut negative);
        assert_eq!(negative, -15);

        let mut zero = 0;
        mutator.apply(&mut zero);
        assert_eq!(zero, 0); // 0 * 3
    }
}

// ============================================================================
// Conditional Stateful Mutator Debug/Display Tests
// ============================================================================
