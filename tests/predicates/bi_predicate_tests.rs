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

    #[test]
    fn test_bi_predicate_not_operator() {
        let boxed = !BoxBiPredicate::new(|x: &i32, y: &i32| x + y > 0);
        assert!(!boxed.test(&5, &3));
        assert!(boxed.test(&-5, &-3));

        let rc = RcBiPredicate::new(|x: &i32, y: &i32| x + y > 0);
        let negated_rc = !&rc;
        assert!(!negated_rc.test(&5, &3));
        assert!(negated_rc.test(&-5, &-3));
        assert!(rc.test(&5, &3));

        let arc = ArcBiPredicate::new(|x: &i32, y: &i32| x + y > 0);
        let negated_arc = !&arc;
        assert!(!negated_arc.test(&5, &3));
        assert!(negated_arc.test(&-5, &-3));
        assert!(arc.test(&5, &3));
    }

    mod bi_predicate_trait_tests {
        use super::BiPredicate;

        #[test]
        fn test_closure_implements_bi_predicate() {
            let sum_positive = |x: &i32, y: &i32| x + y > 0;
            assert!(sum_positive.test(&5, &3));
            assert!(!sum_positive.test(&-5, &-3));
            assert!(!sum_positive.test(&5, &-10));
        }

        #[test]
        fn test_function_pointer_implements_bi_predicate() {
            fn first_greater_than_second(x: &i32, y: &i32) -> bool {
                x > y
            }

            assert!(first_greater_than_second.test(&10, &5));
            assert!(!first_greater_than_second.test(&3, &8));
        }

        #[test]
        fn test_bi_predicate_with_different_types() {
            // Test with different types
            let str_length_greater = |s: &String, len: &usize| s.len() > *len;
            assert!(str_length_greater.test(&String::from("hello"), &3));
            assert!(!str_length_greater.test(&String::from("hi"), &5));

            // Test with mixed types
            let contains_prefix = |s: &&str, prefix: &&str| s.starts_with(*prefix);
            assert!(contains_prefix.test(&"hello", &"hel"));
            assert!(!contains_prefix.test(&"world", &"wor1"));

            // Test with numeric types
            let within_range = |value: &f64, max: &f64| *value <= *max && *value >= 0.0;
            assert!(within_range.test(&5.5, &10.0));
            assert!(!within_range.test(&15.5, &10.0));
        }

        #[test]
        fn test_bi_predicate_with_same_type() {
            let both_positive = |x: &i32, y: &i32| *x > 0 && *y > 0;
            assert!(both_positive.test(&5, &3));
            assert!(!both_positive.test(&-5, &3));
            assert!(!both_positive.test(&5, &-3));
        }
    }

    // ========================================================================
    // Concrete wrapper composition tests
    // ========================================================================

    mod box_bi_predicate_tests {
        use super::BiPredicate;
        use super::BoxBiPredicate;

        #[test]
        fn test_new() {
            let pred = BoxBiPredicate::new(|x: &i32, y: &i32| x + y > 0);
            assert!(pred.test(&5, &3));
            assert!(!pred.test(&-5, &-3));
        }

        #[test]
        fn test_with_name() {
            let pred = BoxBiPredicate::new_with_name("sum_positive", |x: &i32, y: &i32| x + y > 0);

            assert_eq!(pred.name(), Some("sum_positive"));
            assert!(pred.test(&5, &3));
        }

        #[test]
        fn test_always_true() {
            let pred: BoxBiPredicate<i32, i32> = BoxBiPredicate::always_true();
            assert_eq!(pred.name(), Some("always_true"));
            assert!(pred.test(&5, &3));
            assert!(pred.test(&-5, &-3));
            assert!(pred.test(&0, &0));
            assert!(pred.test(&100, &-100));
        }

        #[test]
        fn test_always_false() {
            let pred: BoxBiPredicate<i32, i32> = BoxBiPredicate::always_false();
            assert_eq!(pred.name(), Some("always_false"));
            assert!(!pred.test(&5, &3));
            assert!(!pred.test(&-5, &-3));
            assert!(!pred.test(&0, &0));
            assert!(!pred.test(&100, &-100));
        }

        #[test]
        fn test_always_true_with_composition() {
            let always_true: BoxBiPredicate<i32, i32> = BoxBiPredicate::always_true();
            let positive_sum = BoxBiPredicate::new(|x: &i32, y: &i32| x + y > 0);

            // always_true AND something = something
            let combined = always_true.and(positive_sum);
            assert!(combined.test(&5, &3));
            assert!(!combined.test(&-5, &-3));
        }

        #[test]
        fn test_always_false_with_composition() {
            let always_false: BoxBiPredicate<i32, i32> = BoxBiPredicate::always_false();
            let positive_sum = BoxBiPredicate::new(|x: &i32, y: &i32| x + y > 0);

            // always_false OR something = something
            let combined = always_false.or(positive_sum);
            assert!(combined.test(&5, &3));
            assert!(!combined.test(&-5, &-3));
        }

        #[test]
        fn test_name_none() {
            let pred = BoxBiPredicate::new(|x: &i32, y: &i32| x + y > 0);
            assert_eq!(pred.name(), None);
        }

        #[test]
        fn test_test_method() {
            let pred = BoxBiPredicate::new(|x: &i32, y: &i32| *x > *y);
            assert!(pred.test(&10, &5));
            assert!(!pred.test(&3, &8));
            assert!(!pred.test(&5, &5));
        }

        #[test]
        fn test_and() {
            let sum_positive = BoxBiPredicate::new(|x: &i32, y: &i32| x + y > 0);
            let first_positive = |x: &i32, _y: &i32| *x > 0;

            let combined = sum_positive.and(first_positive);
            assert!(combined.test(&5, &3));
            assert!(!combined.test(&-5, &10));
        }

        #[test]
        fn test_or() {
            let sum_positive = BoxBiPredicate::new(|x: &i32, y: &i32| x + y > 0);
            let first_positive = BoxBiPredicate::new(|x: &i32, _y: &i32| *x > 0);

            let combined = sum_positive.or(first_positive);
            assert!(combined.test(&5, &3));
            assert!(combined.test(&-5, &10));
            assert!(combined.test(&5, &-10));
        }

        #[test]
        fn test_not() {
            let sum_positive = BoxBiPredicate::new(|x: &i32, y: &i32| x + y > 0);
            let sum_not_positive = !sum_positive;

            assert!(!sum_not_positive.test(&5, &3));
            assert!(sum_not_positive.test(&-5, &-3));
        }

        #[test]
        fn test_xor() {
            let first_positive = BoxBiPredicate::new(|x: &i32, _y: &i32| *x > 0);
            let second_positive = BoxBiPredicate::new(|_x: &i32, y: &i32| *y > 0);

            let combined = first_positive.xor(second_positive);
            assert!(combined.test(&5, &-3));
            assert!(combined.test(&-5, &3));
            assert!(!combined.test(&5, &3));
        }

        #[test]
        fn test_nand() {
            let first_positive = BoxBiPredicate::new(|x: &i32, _y: &i32| *x > 0);
            let second_positive = BoxBiPredicate::new(|_x: &i32, y: &i32| *y > 0);

            let combined = first_positive.nand(second_positive);
            assert!(!combined.test(&5, &3));
            assert!(combined.test(&5, &-3));
        }

        #[test]
        fn test_nor() {
            let first_positive = BoxBiPredicate::new(|x: &i32, _y: &i32| *x > 0);
            let second_positive = BoxBiPredicate::new(|_x: &i32, y: &i32| *y > 0);

            let combined = first_positive.nor(second_positive);
            assert!(!combined.test(&5, &3));
            assert!(combined.test(&-5, &-3));
        }

        #[test]
        fn test_chain_combination() {
            let x_positive = BoxBiPredicate::new(|x: &i32, _y: &i32| *x > 0);
            let y_positive = BoxBiPredicate::new(|_x: &i32, y: &i32| *y > 0);

            let both_positive = x_positive.and(y_positive);
            assert!(both_positive.test(&5, &3));
            assert!(!both_positive.test(&5, &-3));
        }

        #[test]
        fn test_display() {
            let pred = BoxBiPredicate::new_with_name("sum_positive", |x: &i32, y: &i32| x + y > 0);
            let display_str = format!("{}", pred);
            assert_eq!(display_str, "BoxBiPredicate(sum_positive)");

            let unnamed = BoxBiPredicate::new(|x: &i32, y: &i32| x + y > 0);
            assert_eq!(format!("{}", unnamed), "BoxBiPredicate(unnamed)");
        }

        #[test]
        fn test_debug() {
            let pred = BoxBiPredicate::new_with_name("test_pred", |x: &i32, y: &i32| x + y > 0);
            let debug_str = format!("{:?}", pred);
            assert!(debug_str.contains("BoxBiPredicate"));
            assert!(debug_str.contains("test_pred"));
        }

        #[test]
        fn test_with_different_types() {
            let str_len_greater = BoxBiPredicate::new(|s: &String, len: &usize| s.len() > *len);
            assert!(str_len_greater.test(&String::from("hello"), &3));
            assert!(!str_len_greater.test(&String::from("hi"), &5));
        }

        #[test]
        fn test_and_with_closure() {
            let sum_positive = BoxBiPredicate::new(|x: &i32, y: &i32| x + y > 0);
            let combined = sum_positive.and(|x: &i32, _y: &i32| *x > 0);
            assert!(combined.test(&5, &3));
            assert!(!combined.test(&-5, &10));
        }

        #[test]
        fn test_set_name() {
            let mut pred = BoxBiPredicate::new(|x: &i32, y: &i32| x + y > 0);
            assert_eq!(pred.name(), None);

            pred.set_name("sum_positive");
            assert_eq!(pred.name(), Some("sum_positive"));
            assert!(pred.test(&5, &3));

            pred.set_name("updated_name");
            assert_eq!(pred.name(), Some("updated_name"));
        }
    }

    // ========================================================================
    // ArcBiPredicate Tests
    // ========================================================================
}
