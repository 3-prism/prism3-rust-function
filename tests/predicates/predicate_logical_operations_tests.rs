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
mod logical_operations_tests {
    use super::ArcPredicate;
    use super::BoxPredicate;
    use super::Predicate;
    use super::RcPredicate;

    // BoxPredicate NAND tests
    #[test]
    fn test_box_nand_basic() {
        let is_positive = BoxPredicate::new(|x: &i32| *x > 0);
        let is_even = BoxPredicate::new(|x: &i32| x % 2 == 0);

        let nand = is_positive.nand(is_even);

        // NAND: true unless both are true
        assert!(nand.test(&3)); // positive but odd: true && false = false, !false = true
        assert!(nand.test(&-2)); // negative but even: false && true = false, !false = true
        assert!(nand.test(&-1)); // negative and odd: false && false = false, !false = true
        assert!(!nand.test(&4)); // positive and even: true && true = true, !true = false
    }

    // BoxPredicate XOR tests
    #[test]
    fn test_box_xor_basic() {
        let is_positive = BoxPredicate::new(|x: &i32| *x > 0);
        let is_even = BoxPredicate::new(|x: &i32| x % 2 == 0);

        let xor = is_positive.xor(is_even);

        // XOR: true if exactly one is true
        assert!(xor.test(&3)); // positive but odd: true ^ false = true
        assert!(xor.test(&-2)); // negative but even: false ^ true = true
        assert!(!xor.test(&-1)); // negative and odd: false ^ false = false
        assert!(!xor.test(&4)); // positive and even: true ^ true = false
    }

    // BoxPredicate NOR tests
    #[test]
    fn test_box_nor_basic() {
        let is_positive = BoxPredicate::new(|x: &i32| *x > 0);
        let is_even = BoxPredicate::new(|x: &i32| x % 2 == 0);

        let nor = is_positive.nor(is_even);

        // NOR: true only when both are false
        assert!(nor.test(&-3)); // negative and odd: !(false || false) = true
        assert!(!nor.test(&3)); // positive but odd: !(true || false) = false
        assert!(!nor.test(&-2)); // negative but even: !(false || true) = false
        assert!(!nor.test(&4)); // positive and even: !(true || true) = false
    }

    // RcPredicate NAND tests
    #[test]
    fn test_rc_nand_basic() {
        let is_positive = RcPredicate::new(|x: &i32| *x > 0);
        let is_even = RcPredicate::new(|x: &i32| x % 2 == 0);

        let nand = is_positive.nand(is_even.clone());

        assert!(nand.test(&3)); // positive but odd
        assert!(nand.test(&-2)); // negative but even
        assert!(nand.test(&-1)); // negative and odd
        assert!(!nand.test(&4)); // positive and even

        // Original predicates still usable
        assert!(is_positive.test(&5));
        assert!(is_even.test(&6));
    }

    // RcPredicate XOR tests
    #[test]
    fn test_rc_xor_basic() {
        let is_positive = RcPredicate::new(|x: &i32| *x > 0);
        let is_even = RcPredicate::new(|x: &i32| x % 2 == 0);

        let xor = is_positive.xor(is_even.clone());

        assert!(xor.test(&3)); // positive but odd
        assert!(xor.test(&-2)); // negative but even
        assert!(!xor.test(&-1)); // negative and odd
        assert!(!xor.test(&4)); // positive and even

        // Original predicates still usable
        assert!(is_positive.test(&5));
        assert!(is_even.test(&6));
    }

    // RcPredicate NOR tests
    #[test]
    fn test_rc_nor_basic() {
        let is_positive = RcPredicate::new(|x: &i32| *x > 0);
        let is_even = RcPredicate::new(|x: &i32| x % 2 == 0);

        let nor = is_positive.nor(is_even.clone());

        // NOR: true only when both are false
        assert!(nor.test(&-3)); // negative and odd: !(false || false) = true
        assert!(!nor.test(&3)); // positive but odd: !(true || false) = false
        assert!(!nor.test(&-2)); // negative but even: !(false || true) = false
        assert!(!nor.test(&4)); // positive and even: !(true || true) = false

        // Original predicates still usable
        assert!(is_positive.test(&5));
        assert!(is_even.test(&6));
    }

    // ArcPredicate NAND tests
    #[test]
    fn test_arc_nand_basic() {
        let is_positive = ArcPredicate::new(|x: &i32| *x > 0);
        let is_even = ArcPredicate::new(|x: &i32| x % 2 == 0);

        let nand = is_positive.nand(is_even.clone());

        assert!(nand.test(&3)); // positive but odd
        assert!(nand.test(&-2)); // negative but even
        assert!(nand.test(&-1)); // negative and odd
        assert!(!nand.test(&4)); // positive and even

        // Original predicates still usable
        assert!(is_positive.test(&5));
        assert!(is_even.test(&6));
    }

    // ArcPredicate XOR tests
    #[test]
    fn test_arc_xor_basic() {
        let is_positive = ArcPredicate::new(|x: &i32| *x > 0);
        let is_even = ArcPredicate::new(|x: &i32| x % 2 == 0);

        let xor = is_positive.xor(is_even.clone());

        assert!(xor.test(&3)); // positive but odd
        assert!(xor.test(&-2)); // negative but even
        assert!(!xor.test(&-1)); // negative and odd
        assert!(!xor.test(&4)); // positive and even

        // Original predicates still usable
        assert!(is_positive.test(&5));
        assert!(is_even.test(&6));
    }

    // ArcPredicate NOR tests
    #[test]
    fn test_arc_nor_basic() {
        let is_positive = ArcPredicate::new(|x: &i32| *x > 0);
        let is_even = ArcPredicate::new(|x: &i32| x % 2 == 0);

        let nor = is_positive.nor(is_even.clone());

        // NOR: true only when both are false
        assert!(nor.test(&-3)); // negative and odd: !(false || false) = true
        assert!(!nor.test(&3)); // positive but odd: !(true || false) = false
        assert!(!nor.test(&-2)); // negative but even: !(false || true) = false
        assert!(!nor.test(&4)); // positive and even: !(true || true) = false

        // Original predicates still usable
        assert!(is_positive.test(&5));
        assert!(is_even.test(&6));
    }

    // Box wrapper NAND tests

    // Box wrapper XOR tests

    // Box wrapper NOR tests

    // Complex composition with NAND

    // Complex composition with XOR

    // NAND with string predicates
    #[test]
    fn test_nand_with_strings() {
        let is_long = BoxPredicate::new(|s: &String| s.len() > 5);
        let has_uppercase = BoxPredicate::new(|s: &String| s.chars().any(|c| c.is_uppercase()));

        let nand = is_long.nand(has_uppercase);

        assert!(nand.test(&"hello".to_string())); // short, no uppercase: !(false && false) = true
        assert!(nand.test(&"Hello".to_string())); // short, has uppercase: !(false && true) = true
        assert!(nand.test(&"goodbye".to_string())); // long, no uppercase: !(true && false) = true
        assert!(!nand.test(&"HelloWorld".to_string())); // long, has uppercase: !(true && true) = false
    }

    // XOR with string predicates
    #[test]
    fn test_xor_with_strings() {
        let is_long = BoxPredicate::new(|s: &String| s.len() > 5);
        let has_uppercase = BoxPredicate::new(|s: &String| s.chars().any(|c| c.is_uppercase()));

        let xor = is_long.xor(has_uppercase);

        assert!(!xor.test(&"hello".to_string())); // short, no uppercase: false ^ false = false
        assert!(xor.test(&"Hello".to_string())); // short, has uppercase: false ^ true = true
        assert!(xor.test(&"goodbye".to_string())); // long, no uppercase: true ^ false = true
        assert!(!xor.test(&"HelloWorld".to_string())); // long, has uppercase: true ^ true = false
    }

    // NOR with string predicates
    #[test]
    fn test_nor_with_strings() {
        let is_long = BoxPredicate::new(|s: &String| s.len() > 5);
        let has_uppercase = BoxPredicate::new(|s: &String| s.chars().any(|c| c.is_uppercase()));

        let nor = is_long.nor(has_uppercase);

        assert!(nor.test(&"hello".to_string())); // short, no uppercase: !(false || false) = true
        assert!(!nor.test(&"Hello".to_string())); // short, has uppercase: !(false || true) = false
        assert!(!nor.test(&"goodbye".to_string())); // long, no uppercase: !(true || false) = false
        assert!(!nor.test(&"HelloWorld".to_string())); // long, has uppercase: !(true || true) = false
    }
}
