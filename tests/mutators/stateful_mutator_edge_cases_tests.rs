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
mod test_edge_cases {
    use super::BoxStatefulMutator;
    use super::StatefulMutator;

    #[test]
    fn test_with_zero() {
        let mut mutator = BoxStatefulMutator::new(|x: &mut i32| *x += 1);
        let mut value = 0;
        mutator.apply(&mut value);
        assert_eq!(value, 1);
    }

    #[test]
    fn test_with_negative() {
        let mut mutator = BoxStatefulMutator::new(|x: &mut i32| *x = x.abs());
        let mut value = -42;
        mutator.apply(&mut value);
        assert_eq!(value, 42);
    }

    #[test]
    fn test_with_max_value() {
        let mut mutator = BoxStatefulMutator::new(|x: &mut i32| *x = x.saturating_add(1));
        let mut value = i32::MAX;
        mutator.apply(&mut value);
        assert_eq!(value, i32::MAX);
    }

    #[test]
    fn test_with_min_value() {
        let mut mutator = BoxStatefulMutator::new(|x: &mut i32| *x = x.saturating_sub(1));
        let mut value = i32::MIN;
        mutator.apply(&mut value);
        assert_eq!(value, i32::MIN);
    }

    #[test]
    fn test_with_empty_string() {
        let mut mutator = BoxStatefulMutator::new(|s: &mut String| s.push_str("added"));
        let mut text = String::new();
        mutator.apply(&mut text);
        assert_eq!(text, "added");
    }

    #[test]
    fn test_with_empty_vec() {
        let mut mutator = BoxStatefulMutator::new(|v: &mut Vec<i32>| v.push(1));
        let mut numbers = Vec::new();
        mutator.apply(&mut numbers);
        assert_eq!(numbers, vec![1]);
    }

    #[test]
    fn test_unicode() {
        let mut mutator = BoxStatefulMutator::new(|s: &mut String| *s = s.to_uppercase());
        let mut text = String::from("héllo world");
        mutator.apply(&mut text);
        assert_eq!(text, "HÉLLO WORLD");
    }
}

// ============================================================================
// Custom StatefulMutator with Default into_xxx() Implementation Tests
// ============================================================================
