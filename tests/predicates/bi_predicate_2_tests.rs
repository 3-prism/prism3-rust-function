// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

#[cfg(test)]
mod tests {
    use std::thread;

    use qubit_function::predicates::ArcBiPredicate;
    use qubit_function::predicates::BiPredicate;
    use qubit_function::predicates::BoxBiPredicate;
    use qubit_function::predicates::RcBiPredicate;

    // ========================================================================
    // BiPredicate Trait Tests - Test closure and function pointer
    // implementations
    // ========================================================================

    mod bi_predicate_ext_tests {
        use super::BiPredicate;
    }

    // ========================================================================
    // BoxBiPredicate Tests
    // ========================================================================

    mod arc_bi_predicate_tests {
        use super::ArcBiPredicate;
        use super::BiPredicate;
        use super::thread;

        #[test]
        fn test_new() {
            let pred = ArcBiPredicate::new(|x: &i32, y: &i32| x + y > 0);
            assert!(pred.test(&5, &3));
            assert!(!pred.test(&-5, &-3));
        }

        #[test]
        fn test_with_name() {
            let pred = ArcBiPredicate::new_with_name("sum_positive", |x: &i32, y: &i32| x + y > 0);

            assert_eq!(pred.name(), Some("sum_positive"));
            assert!(pred.test(&5, &3));
        }

        #[test]
        fn test_always_true() {
            let pred: ArcBiPredicate<i32, i32> = ArcBiPredicate::always_true();
            assert_eq!(pred.name(), Some("always_true"));
            assert!(pred.test(&5, &3));
            assert!(pred.test(&-5, &-3));
            assert!(pred.test(&0, &0));
            assert!(pred.test(&100, &-100));
        }

        #[test]
        fn test_always_false() {
            let pred: ArcBiPredicate<i32, i32> = ArcBiPredicate::always_false();
            assert_eq!(pred.name(), Some("always_false"));
            assert!(!pred.test(&5, &3));
            assert!(!pred.test(&-5, &-3));
            assert!(!pred.test(&0, &0));
            assert!(!pred.test(&100, &-100));
        }

        #[test]
        fn test_always_true_with_composition() {
            let always_true: ArcBiPredicate<i32, i32> = ArcBiPredicate::always_true();
            let positive_sum = |x: &i32, y: &i32| x + y > 0;

            // always_true AND something = something
            let combined = always_true.and(positive_sum);
            assert!(combined.test(&5, &3));
            assert!(!combined.test(&-5, &-3));
        }

        #[test]
        fn test_always_false_with_composition() {
            let always_false: ArcBiPredicate<i32, i32> = ArcBiPredicate::always_false();
            let positive_sum = |x: &i32, y: &i32| x + y > 0;

            // always_false OR something = something
            let combined = always_false.or(positive_sum);
            assert!(combined.test(&5, &3));
            assert!(!combined.test(&-5, &-3));
        }

        #[test]
        fn test_always_true_clone() {
            let pred: ArcBiPredicate<i32, i32> = ArcBiPredicate::always_true();
            let cloned = pred.clone();

            assert_eq!(cloned.name(), Some("always_true"));
            assert!(cloned.test(&5, &3));
            assert!(pred.test(&-5, &-3));
        }

        #[test]
        fn test_name_none() {
            let pred = ArcBiPredicate::new(|x: &i32, y: &i32| x + y > 0);
            assert_eq!(pred.name(), None);
        }

        #[test]
        fn test_test_method() {
            let pred = ArcBiPredicate::new(|x: &i32, y: &i32| *x > *y);
            assert!(pred.test(&10, &5));
            assert!(!pred.test(&3, &8));
        }

        #[test]
        fn test_clone() {
            let pred = ArcBiPredicate::new(|x: &i32, y: &i32| x + y > 0);
            let cloned = pred.clone();

            assert!(pred.test(&5, &3));
            assert!(cloned.test(&5, &3));
            assert!(!pred.test(&-5, &-3));
            assert!(!cloned.test(&-5, &-3));
        }

        #[test]
        fn test_clone_preserves_name() {
            let pred = ArcBiPredicate::new_with_name("original", |x: &i32, y: &i32| x + y > 0);
            let cloned = pred.clone();

            assert_eq!(pred.name(), Some("original"));
            assert_eq!(cloned.name(), Some("original"));
        }

        #[test]
        fn test_and() {
            let sum_positive = ArcBiPredicate::new(|x: &i32, y: &i32| x + y > 0);
            let first_positive = ArcBiPredicate::new(|x: &i32, _y: &i32| *x > 0);

            let combined = sum_positive.clone().and(first_positive.clone());
            assert!(combined.test(&5, &3));
            assert!(!combined.test(&-5, &10));

            // Original predicates still usable
            assert!(sum_positive.test(&-5, &10));
            assert!(first_positive.test(&5, &-10));
        }

        #[test]
        fn test_or() {
            let sum_positive = ArcBiPredicate::new(|x: &i32, y: &i32| x + y > 0);
            let first_positive = ArcBiPredicate::new(|x: &i32, _y: &i32| *x > 0);

            let combined = sum_positive.or(first_positive.clone());
            assert!(combined.test(&5, &3));
            assert!(combined.test(&-5, &10));
        }

        #[test]
        fn test_not() {
            let sum_positive = ArcBiPredicate::new(|x: &i32, y: &i32| x + y > 0);
            let sum_not_positive = !&sum_positive;

            assert!(!sum_not_positive.test(&5, &3));
            assert!(sum_not_positive.test(&-5, &-3));

            // Original still usable
            assert!(sum_positive.test(&5, &3));
        }

        #[test]
        fn test_xor() {
            let first_positive = ArcBiPredicate::new(|x: &i32, _y: &i32| *x > 0);
            let second_positive = ArcBiPredicate::new(|_x: &i32, y: &i32| *y > 0);

            let combined = first_positive.xor(second_positive);
            assert!(combined.test(&5, &-3));
            assert!(!combined.test(&5, &3));
        }

        #[test]
        fn test_nand() {
            let first_positive = ArcBiPredicate::new(|x: &i32, _y: &i32| *x > 0);
            let second_positive = ArcBiPredicate::new(|_x: &i32, y: &i32| *y > 0);

            let combined = first_positive.nand(second_positive);
            assert!(!combined.test(&5, &3));
            assert!(combined.test(&5, &-3));
        }

        #[test]
        fn test_nor() {
            let first_positive = ArcBiPredicate::new(|x: &i32, _y: &i32| *x > 0);
            let second_positive = ArcBiPredicate::new(|_x: &i32, y: &i32| *y > 0);

            let combined = first_positive.nor(second_positive);
            assert!(!combined.test(&5, &3));
            assert!(combined.test(&-5, &-3));
        }

        #[test]
        fn test_chain_combination() {
            let x_positive = ArcBiPredicate::new(|x: &i32, _y: &i32| *x > 0);
            let y_positive = ArcBiPredicate::new(|_x: &i32, y: &i32| *y > 0);
            let sum_large = ArcBiPredicate::new(|x: &i32, y: &i32| x + y > 100);

            let complex = x_positive.and(y_positive).or(sum_large);
            assert!(complex.test(&5, &3)); // Both positive
            assert!(complex.test(&50, &60)); // Sum large
        }

        #[test]
        fn test_display() {
            let pred = ArcBiPredicate::new_with_name("sum_positive", |x: &i32, y: &i32| x + y > 0);
            assert_eq!(format!("{}", pred), "ArcBiPredicate(sum_positive)");
        }

        #[test]
        fn test_debug() {
            let pred = ArcBiPredicate::new_with_name("test_pred", |x: &i32, y: &i32| x + y > 0);
            let debug_str = format!("{:?}", pred);
            assert!(debug_str.contains("ArcBiPredicate"));
        }

        #[test]
        fn test_with_different_types() {
            let str_len_greater = ArcBiPredicate::new(|s: &String, len: &usize| s.len() > *len);
            assert!(str_len_greater.test(&String::from("hello"), &3));
            assert!(!str_len_greater.test(&String::from("hi"), &5));
        }

        #[test]
        fn test_set_name() {
            let mut pred = ArcBiPredicate::new(|x: &i32, y: &i32| x + y > 0);
            assert_eq!(pred.name(), None);

            pred.set_name("sum_positive");
            assert_eq!(pred.name(), Some("sum_positive"));
            assert!(pred.test(&5, &3));

            pred.set_name("updated_name");
            assert_eq!(pred.name(), Some("updated_name"));
        }

        #[test]
        fn test_thread_safety() {
            let pred = ArcBiPredicate::new(|x: &i32, y: &i32| x + y > 0);

            let pred_clone = pred.clone();
            let handle = thread::spawn(move || pred_clone.test(&5, &3));

            assert!(pred.test(&10, &-5));
            assert!(handle.join().expect("thread should not panic"));
        }

        #[test]
        fn test_multiple_threads() {
            let pred = ArcBiPredicate::new(|x: &i32, y: &i32| x + y > 100);

            let handles: Vec<_> = (0..10)
                .map(|i| {
                    let pred_clone = pred.clone();
                    thread::spawn(move || pred_clone.test(&(i * 10), &20))
                })
                .collect();

            for handle in handles {
                let _ = handle.join().expect("thread should not panic");
            }
        }
    }

    // ========================================================================
    // RcBiPredicate Tests
    // ========================================================================
}
