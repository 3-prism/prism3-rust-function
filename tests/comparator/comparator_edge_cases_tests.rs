// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
#![cfg(feature = "full")]
use std::cmp::Ordering;

use qubit_function::comparator::ArcComparator;
use qubit_function::comparator::BoxComparator;
use qubit_function::comparator::Comparator;
use qubit_function::comparator::RcComparator;

#[cfg(test)]
mod edge_cases {
    use super::BoxComparator;
    use super::Comparator;
    use super::Ordering;

    #[test]
    fn test_with_empty_values() {
        let cmp = BoxComparator::new(|a: &String, b: &String| a.cmp(b));
        assert_eq!(cmp.compare(&String::new(), &"hello".to_string()), Ordering::Less);
    }

    #[test]
    fn test_with_negative_numbers() {
        let cmp = BoxComparator::new(|a: &i32, b: &i32| a.cmp(b));
        assert_eq!(cmp.compare(&-5, &-3), Ordering::Less);
        assert_eq!(cmp.compare(&-3, &-5), Ordering::Greater);
    }

    #[test]
    fn test_multiple_reversals() {
        let cmp = BoxComparator::new(|a: &i32, b: &i32| a.cmp(b));
        let rev1 = cmp.reversed();
        let rev2 = rev1.reversed();
        // Double reversal should be same as original
        assert_eq!(rev2.compare(&5, &3), Ordering::Greater);
    }

    #[test]
    fn test_long_chain() {
        let cmp1 = BoxComparator::new(|a: &i32, b: &i32| (a / 10).cmp(&(b / 10)));
        let cmp2 = BoxComparator::new(|a: &i32, b: &i32| (a % 10).cmp(&(b % 10)));
        let chained = cmp1.then_comparing(cmp2);
        assert_eq!(chained.compare(&15, &12), Ordering::Greater);
        assert_eq!(chained.compare(&12, &15), Ordering::Less);
    }
}
