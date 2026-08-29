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

#[test]
fn test_comparator_default_conversions_allow_relaxed_generic_types() {
    #[derive(Clone, Debug)]
    struct BorrowedRc<'a> {
        value: &'a str,
    }

    #[derive(Clone, Debug)]
    struct BorrowedRcComparator;

    impl<'a> Comparator<BorrowedRc<'a>> for BorrowedRcComparator {
        fn compare(&self, first: &BorrowedRc<'a>, second: &BorrowedRc<'a>) -> Ordering {
            first.value.cmp(second.value)
        }
    }

    let left_text = String::from("left");
    let right_text = String::from("right");
    let left = BorrowedRc {
        value: left_text.as_str(),
    };
    let right = BorrowedRc {
        value: right_text.as_str(),
    };
    let comparator = BorrowedRcComparator;

    assert_eq!(
        BoxComparator::new(comparator.clone()).compare(&left, &right),
        Ordering::Less
    );
    assert_eq!(
        RcComparator::new(comparator.clone()).compare(&left, &right),
        Ordering::Less
    );
    assert_eq!(ArcComparator::new(comparator).compare(&left, &right), Ordering::Less);
}
