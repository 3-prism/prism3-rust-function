// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Unit tests for the predicate module.

use std::cell::RefCell;
use std::sync::Arc;
use std::sync::Mutex;

use qubit_function::predicates::ArcPredicate;
use qubit_function::predicates::BoxPredicate;
use qubit_function::predicates::Predicate;
use qubit_function::predicates::RcPredicate;

struct PositivePredicate;

impl Predicate<i32> for PositivePredicate {
    fn test(&self, value: &i32) -> bool {
        *value > 0
    }
}

#[cfg(test)]
mod always_predicates_tests {
    use super::ArcPredicate;
    use super::BoxPredicate;
    use super::Predicate;
    use super::RcPredicate;

    #[test]
    fn test_box_always_true() {
        let pred = BoxPredicate::<i32>::always_true();
        assert!(pred.test(&5));
        assert!(pred.test(&-5));
        assert!(pred.test(&0));
    }

    #[test]
    fn test_box_always_false() {
        let pred = BoxPredicate::<i32>::always_false();
        assert!(!pred.test(&5));
        assert!(!pred.test(&-5));
        assert!(!pred.test(&0));
    }

    #[test]
    fn test_rc_always_true() {
        let pred = RcPredicate::<i32>::always_true();
        assert!(pred.test(&5));
        assert!(pred.test(&-5));
        assert!(pred.test(&0));
    }

    #[test]
    fn test_rc_always_false() {
        let pred = RcPredicate::<i32>::always_false();
        assert!(!pred.test(&5));
        assert!(!pred.test(&-5));
        assert!(!pred.test(&0));
    }

    #[test]
    fn test_arc_always_true() {
        let pred = ArcPredicate::<i32>::always_true();
        assert!(pred.test(&5));
        assert!(pred.test(&-5));
        assert!(pred.test(&0));
    }

    #[test]
    fn test_arc_always_false() {
        let pred = ArcPredicate::<i32>::always_false();
        assert!(!pred.test(&5));
        assert!(!pred.test(&-5));
        assert!(!pred.test(&0));
    }

    #[test]
    fn test_always_true_with_composition() {
        let always = BoxPredicate::<i32>::always_true();
        let is_positive = |x: &i32| *x > 0;

        let and_result = always.and(is_positive);
        assert!(and_result.test(&5));
        assert!(!and_result.test(&-5));
    }

    #[test]
    fn test_always_false_with_composition() {
        let never = BoxPredicate::<i32>::always_false();
        let is_positive = |x: &i32| *x > 0;

        let or_result = never.or(is_positive);
        assert!(or_result.test(&5));
        assert!(!or_result.test(&-5));
    }

    #[test]
    fn test_new_with_name() {
        let mut pred = BoxPredicate::new_with_name("positive", |x: &i32| *x > 0);
        assert_eq!(pred.name(), Some("positive"));
        assert!(pred.test(&5));

        pred.set_name("updated");
        assert_eq!(pred.name(), Some("updated"));
    }

    #[test]
    fn test_rc_new_with_name() {
        let mut pred = RcPredicate::new_with_name("positive", |x: &i32| *x > 0);
        assert_eq!(pred.name(), Some("positive"));
        assert!(pred.test(&5));

        pred.set_name("updated");
        assert_eq!(pred.name(), Some("updated"));
    }

    #[test]
    fn test_arc_new_with_name() {
        let mut pred = ArcPredicate::new_with_name("positive", |x: &i32| *x > 0);
        assert_eq!(pred.name(), Some("positive"));
        assert!(pred.test(&5));

        pred.set_name("updated");
        assert_eq!(pred.name(), Some("updated"));
    }
}
