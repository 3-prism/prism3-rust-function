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
mod arc_comparator_tests {
    use super::ArcComparator;
    use super::BoxComparator;
    use super::Comparator;
    use super::Ordering;
    use super::RcComparator;

    #[test]
    fn test_new_and_compare() {
        let cmp = ArcComparator::new(|a: &i32, b: &i32| a.cmp(b));
        assert_eq!(cmp.compare(&5, &3), Ordering::Greater);
        assert_eq!(cmp.compare(&3, &5), Ordering::Less);
        assert_eq!(cmp.compare(&5, &5), Ordering::Equal);
    }

    #[test]
    fn test_clone() {
        let cmp = ArcComparator::new(|a: &i32, b: &i32| a.cmp(b));
        let cloned = cmp.clone();
        assert_eq!(cmp.compare(&5, &3), Ordering::Greater);
        assert_eq!(cloned.compare(&5, &3), Ordering::Greater);
    }

    #[test]
    fn test_reversed() {
        let cmp = ArcComparator::new(|a: &i32, b: &i32| a.cmp(b));
        let rev = cmp.reversed();
        assert_eq!(rev.compare(&5, &3), Ordering::Less);
        // Original still works
        assert_eq!(cmp.compare(&5, &3), Ordering::Greater);
    }

    #[test]
    fn test_then_comparing() {
        let cmp1 = ArcComparator::new(|a: &i32, b: &i32| (a % 2).cmp(&(b % 2)));
        let cmp2 = ArcComparator::new(|a: &i32, b: &i32| a.cmp(b));
        let chained = cmp1.then_comparing(cmp2.clone());
        assert_eq!(chained.compare(&4, &2), Ordering::Greater);
        // Originals still work
        assert_eq!(cmp1.compare(&4, &2), Ordering::Equal);
        assert_eq!(cmp2.compare(&4, &2), Ordering::Greater);
    }

    #[test]
    fn test_then_comparing_with_non_equal_greater() {
        // Test the case where the first comparator returns Greater
        let cmp1 = ArcComparator::new(|a: &i32, b: &i32| a.cmp(b));
        let cmp2 = ArcComparator::new(|_a: &i32, _b: &i32| panic!("Second comparator should not be called"));
        let chained = cmp1.then_comparing(cmp2.clone());
        // 5 > 3, so first comparator returns Greater, second not called
        assert_eq!(chained.compare(&5, &3), Ordering::Greater);
    }

    #[test]
    fn test_then_comparing_with_non_equal_less() {
        // Test the case where the first comparator returns Less
        let cmp1 = ArcComparator::new(|a: &i32, b: &i32| a.cmp(b));
        let cmp2 = ArcComparator::new(|_a: &i32, _b: &i32| panic!("Second comparator should not be called"));
        let chained = cmp1.then_comparing(cmp2.clone());
        // 3 < 5, so first comparator returns Less, second not called
        assert_eq!(chained.compare(&3, &5), Ordering::Less);
    }

    #[test]
    fn test_comparing() {
        #[derive(Debug)]
        struct Person {
            name: String,
            age: i32,
        }

        let by_age = ArcComparator::comparing(|p: &Person| &p.age);
        let p1 = Person {
            name: "Alice".to_string(),
            age: 30,
        };
        let p2 = Person {
            name: "Bob".to_string(),
            age: 25,
        };
        assert_eq!(p1.name, "Alice");
        assert_eq!(p2.name, "Bob");
        assert_eq!(by_age.compare(&p1, &p2), Ordering::Greater);
    }

    #[test]
    fn test_into_fn() {
        let cmp = ArcComparator::new(|a: &i32, b: &i32| a.cmp(b));
        let func = move |first: &i32, second: &i32| cmp.compare(first, second);
        assert_eq!(func(&5, &3), Ordering::Greater);
    }

    #[test]
    fn test_into_box() {
        let cmp = ArcComparator::new(|a: &i32, b: &i32| a.cmp(b));
        let boxed = BoxComparator::new(cmp);
        assert_eq!(boxed.compare(&5, &3), Ordering::Greater);
    }

    #[test]
    fn test_into_rc() {
        let cmp = ArcComparator::new(|a: &i32, b: &i32| a.cmp(b));
        let rc = RcComparator::new(cmp);
        assert_eq!(rc.compare(&5, &3), Ordering::Greater);
    }

    #[test]
    fn test_into_arc() {
        let cmp = ArcComparator::new(|a: &i32, b: &i32| a.cmp(b));
        let arc = ArcComparator::new(cmp);
        assert_eq!(arc.compare(&5, &3), Ordering::Greater);
    }

    #[test]
    fn test_thread_safety() {
        let cmp = ArcComparator::new(|a: &i32, b: &i32| a.cmp(b));
        let handles: Vec<_> = (0..10)
            .map(|i| {
                let cmp_clone = cmp.clone();
                std::thread::spawn(move || {
                    assert_eq!(cmp_clone.compare(&(i + 1), &i), Ordering::Greater);
                })
            })
            .collect();

        for handle in handles {
            handle.join().expect("thread should not panic");
        }
    }
}
