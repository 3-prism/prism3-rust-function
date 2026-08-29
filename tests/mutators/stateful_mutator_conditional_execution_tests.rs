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
    fn test_box_when_with_closure() {
        let mut mutator = BoxStatefulMutator::new(|x: &mut i32| *x *= 2).when(|x: &i32| *x > 0);

        let mut positive = 5;
        mutator.apply(&mut positive);
        assert_eq!(positive, 10);

        let mut negative = -5;
        mutator.apply(&mut negative);
        assert_eq!(negative, -5);
    }

    #[test]
    fn test_box_when_with_function_pointer() {
        let mut mutator = BoxStatefulMutator::new(|x: &mut i32| *x *= 2).when(is_positive as fn(&i32) -> bool);

        let mut positive = 5;
        mutator.apply(&mut positive);
        assert_eq!(positive, 10);

        let mut negative = -5;
        mutator.apply(&mut negative);
        assert_eq!(negative, -5);
    }

    #[test]
    fn test_box_when_with_box_predicate() {
        let pred = BoxPredicate::new(|x: &i32| *x > 0);
        let mut mutator = BoxStatefulMutator::new(|x: &mut i32| *x *= 2).when(pred);

        let mut positive = 5;
        mutator.apply(&mut positive);
        assert_eq!(positive, 10);

        let mut negative = -5;
        mutator.apply(&mut negative);
        assert_eq!(negative, -5);
    }

    #[test]
    fn test_box_when_with_rc_predicate() {
        let pred = RcPredicate::new(|x: &i32| *x > 0);
        let mut mutator = BoxStatefulMutator::new(|x: &mut i32| *x *= 2).when(pred);

        let mut positive = 5;
        mutator.apply(&mut positive);
        assert_eq!(positive, 10);

        let mut negative = -5;
        mutator.apply(&mut negative);
        assert_eq!(negative, -5);
    }

    #[test]
    fn test_box_when_with_arc_predicate() {
        let pred = ArcPredicate::new(|x: &i32| *x > 0);
        let mut mutator = BoxStatefulMutator::new(|x: &mut i32| *x *= 2).when(pred);

        let mut positive = 5;
        mutator.apply(&mut positive);
        assert_eq!(positive, 10);

        let mut negative = -5;
        mutator.apply(&mut negative);
        assert_eq!(negative, -5);
    }

    // ========================================================================
    // BoxConditionalStatefulMutator::or_else() tests
    // ========================================================================

    #[test]
    fn test_box_or_else_with_closure() {
        let mut mutator = BoxStatefulMutator::new(|x: &mut i32| *x *= 2)
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
    fn test_box_or_else_with_function_pointer() {
        let mut mutator = BoxStatefulMutator::new(|x: &mut i32| *x *= 2)
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
    fn test_box_or_else_with_box_mutator() {
        let else_mutator = BoxStatefulMutator::new(|x: &mut i32| *x = 0);
        let mut mutator = BoxStatefulMutator::new(|x: &mut i32| *x *= 2)
            .when(|x: &i32| *x > 0)
            .or_else(else_mutator);

        let mut positive = 5;
        mutator.apply(&mut positive);
        assert_eq!(positive, 10);

        let mut negative = -5;
        mutator.apply(&mut negative);
        assert_eq!(negative, 0);
    }

    #[test]
    fn test_box_or_else_with_rc_mutator() {
        let else_mutator = RcStatefulMutator::new(|x: &mut i32| *x = 100);
        let mut mutator = BoxStatefulMutator::new(|x: &mut i32| *x *= 2)
            .when(|x: &i32| *x > 0)
            .or_else(else_mutator);

        let mut positive = 5;
        mutator.apply(&mut positive);
        assert_eq!(positive, 10);

        let mut negative = -5;
        mutator.apply(&mut negative);
        assert_eq!(negative, 100);
    }

    #[test]
    fn test_box_or_else_with_arc_mutator() {
        let else_mutator = ArcStatefulMutator::new(|x: &mut i32| *x = 200);
        let mut mutator = BoxStatefulMutator::new(|x: &mut i32| *x *= 2)
            .when(|x: &i32| *x > 0)
            .or_else(else_mutator);

        let mut positive = 5;
        mutator.apply(&mut positive);
        assert_eq!(positive, 10);

        let mut negative = -5;
        mutator.apply(&mut negative);
        assert_eq!(negative, 200);
    }

    // ========================================================================
    // BoxConditionalStatefulMutator::and_then() tests
    // ========================================================================

    #[test]
    fn test_box_conditional_and_then_with_closure() {
        let cond1 = BoxStatefulMutator::new(|x: &mut i32| *x *= 2).when(|x: &i32| *x > 0);
        let mut chained = cond1.and_then(|x: &mut i32| *x += 10);

        let mut positive = 5;
        chained.apply(&mut positive);
        assert_eq!(positive, 20); // 5 * 2 + 10

        let mut negative = -5;
        chained.apply(&mut negative);
        assert_eq!(negative, 5); // -5 + 10 (not doubled)
    }

    #[test]
    fn test_box_conditional_and_then_with_box_mutator() {
        let cond1 = BoxStatefulMutator::new(|x: &mut i32| *x *= 2).when(|x: &i32| *x > 0);
        let next = BoxStatefulMutator::new(|x: &mut i32| *x += 100);
        let mut chained = cond1.and_then(next);

        let mut positive = 10;
        chained.apply(&mut positive);
        assert_eq!(positive, 120); // 10 * 2 + 100

        let mut negative = -10;
        chained.apply(&mut negative);
        assert_eq!(negative, 90); // -10 + 100 (not doubled)
    }

    #[test]
    fn test_box_conditional_and_then_conditional() {
        let cond1 = BoxStatefulMutator::new(|x: &mut i32| *x *= 2).when(|x: &i32| *x > 0);
        let cond2 = BoxStatefulMutator::new(|x: &mut i32| *x = 100).when(|x: &i32| *x > 100);
        let mut chained = cond1.and_then(cond2);

        let mut small = 5;
        chained.apply(&mut small);
        assert_eq!(small, 10); // 5 * 2 = 10 (< 100, not capped)

        let mut large = 60;
        chained.apply(&mut large);
        assert_eq!(large, 100); // 60 * 2 = 120 (> 100, capped)
    }

    // ========================================================================
    // RcConditionalStatefulMutator::and_then() tests
    // ========================================================================

    #[test]
    fn test_rc_conditional_and_then_with_closure() {
        let conditional = RcStatefulMutator::new(|x: &mut i32| *x *= 2).when(|x: &i32| *x > 0);
        let mut chained = conditional.and_then(|x: &mut i32| *x += 10);

        let mut positive = 5;
        chained.apply(&mut positive);
        assert_eq!(positive, 20); // 5 * 2 + 10

        let mut negative = -5;
        chained.apply(&mut negative);
        assert_eq!(negative, 5); // -5 + 10 (condition not met)
    }

    #[test]
    fn test_rc_conditional_and_then_with_rc_mutator() {
        let conditional = RcStatefulMutator::new(|x: &mut i32| *x *= 2).when(|x: &i32| *x > 0);
        let next = RcStatefulMutator::new(|x: &mut i32| *x += 100);
        let mut chained = conditional.and_then(next);

        let mut positive = 10;
        chained.apply(&mut positive);
        assert_eq!(positive, 120); // 10 * 2 + 100

        let mut negative = -10;
        chained.apply(&mut negative);
        assert_eq!(negative, 90); // -10 + 100 (condition not met)
    }

    // ========================================================================
    // ArcConditionalStatefulMutator::and_then() tests
    // ========================================================================

    #[test]
    fn test_arc_conditional_and_then_with_closure() {
        let conditional = ArcStatefulMutator::new(|x: &mut i32| *x *= 2).when(|x: &i32| *x > 0);
        let mut chained = conditional.and_then(|x: &mut i32| *x += 10);

        let mut positive = 5;
        chained.apply(&mut positive);
        assert_eq!(positive, 20); // 5 * 2 + 10

        let mut negative = -5;
        chained.apply(&mut negative);
        assert_eq!(negative, 5); // -5 + 10 (condition not met)
    }

    #[test]
    fn test_arc_conditional_and_then_with_arc_mutator() {
        let conditional = ArcStatefulMutator::new(|x: &mut i32| *x *= 2).when(|x: &i32| *x > 0);
        let next = ArcStatefulMutator::new(|x: &mut i32| *x += 100);
        let mut chained = conditional.and_then(next);

        let mut positive = 10;
        chained.apply(&mut positive);
        assert_eq!(positive, 120); // 10 * 2 + 100

        let mut negative = -10;
        chained.apply(&mut negative);
        assert_eq!(negative, 90); // -10 + 100 (condition not met)
    }

    // ========================================================================
    // RcStatefulMutator::when() tests
    // ========================================================================

    #[test]
    fn test_rc_when_with_closure() {
        let conditional = RcStatefulMutator::new(|x: &mut i32| *x *= 2).when(|x: &i32| *x > 0);
        let mut m = conditional.clone();

        let mut positive = 5;
        m.apply(&mut positive);
        assert_eq!(positive, 10);

        let mut negative = -5;
        m.apply(&mut negative);
        assert_eq!(negative, -5);
    }

    #[test]
    fn test_rc_when_with_function_pointer() {
        let conditional = RcStatefulMutator::new(|x: &mut i32| *x *= 2).when(is_positive as fn(&i32) -> bool);
        let mut m = conditional.clone();

        let mut positive = 5;
        m.apply(&mut positive);
        assert_eq!(positive, 10);

        let mut negative = -5;
        m.apply(&mut negative);
        assert_eq!(negative, -5);
    }

    #[test]
    fn test_rc_when_with_rc_predicate() {
        let pred = RcPredicate::new(|x: &i32| *x > 0);
        let conditional = RcStatefulMutator::new(|x: &mut i32| *x *= 2).when(pred);
        let mut m = conditional.clone();

        let mut positive = 5;
        m.apply(&mut positive);
        assert_eq!(positive, 10);

        let mut negative = -5;
        m.apply(&mut negative);
        assert_eq!(negative, -5);
    }

    #[test]
    fn test_rc_when_with_box_predicate() {
        let pred = BoxPredicate::new(|x: &i32| *x > 0);
        let conditional = RcStatefulMutator::new(|x: &mut i32| *x *= 2).when(pred);
        let mut m = conditional.clone();

        let mut positive = 5;
        m.apply(&mut positive);
        assert_eq!(positive, 10);

        let mut negative = -5;
        m.apply(&mut negative);
        assert_eq!(negative, -5);
    }

    // ========================================================================
    // RcConditionalStatefulMutator::or_else() tests
    // ========================================================================

    #[test]
    fn test_rc_or_else_with_closure() {
        let mut mutator = RcStatefulMutator::new(|x: &mut i32| *x *= 2)
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
    fn test_rc_or_else_with_function_pointer() {
        let mut mutator = RcStatefulMutator::new(|x: &mut i32| *x *= 2)
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
    fn test_rc_or_else_with_rc_mutator() {
        let else_mutator = RcStatefulMutator::new(|x: &mut i32| *x = 100);
        let mut mutator = RcStatefulMutator::new(|x: &mut i32| *x *= 2)
            .when(|x: &i32| *x > 0)
            .or_else(else_mutator);

        let mut positive = 5;
        mutator.apply(&mut positive);
        assert_eq!(positive, 10);

        let mut negative = -5;
        mutator.apply(&mut negative);
        assert_eq!(negative, 100);
    }

    #[test]
    fn test_rc_or_else_with_box_mutator() {
        let else_mutator = BoxStatefulMutator::new(|x: &mut i32| *x = 200);
        let mut mutator = RcStatefulMutator::new(|x: &mut i32| *x *= 2)
            .when(|x: &i32| *x > 0)
            .or_else(else_mutator);

        let mut positive = 5;
        mutator.apply(&mut positive);
        assert_eq!(positive, 10);

        let mut negative = -5;
        mutator.apply(&mut negative);
        assert_eq!(negative, 200);
    }

    // ========================================================================
    // RcConditionalStatefulMutator::clone() tests
    // ========================================================================
}

// ============================================================================
// Conditional Stateful Mutator Debug/Display Tests
// ============================================================================
