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

    mod rc_bi_predicate_tests {
        use super::BiPredicate;
        use super::RcBiPredicate;

        #[test]
        fn test_new() {
            let pred = RcBiPredicate::new(|x: &i32, y: &i32| x + y > 0);
            assert!(pred.test(&5, &3));
            assert!(!pred.test(&-5, &-3));
        }

        #[test]
        fn test_with_name() {
            let pred = RcBiPredicate::new_with_name("sum_positive", |x: &i32, y: &i32| x + y > 0);

            assert_eq!(pred.name(), Some("sum_positive"));
            assert!(pred.test(&5, &3));
        }

        #[test]
        fn test_always_true() {
            let pred: RcBiPredicate<i32, i32> = RcBiPredicate::always_true();
            assert_eq!(pred.name(), Some("always_true"));
            assert!(pred.test(&5, &3));
            assert!(pred.test(&-5, &-3));
            assert!(pred.test(&0, &0));
            assert!(pred.test(&100, &-100));
        }

        #[test]
        fn test_always_false() {
            let pred: RcBiPredicate<i32, i32> = RcBiPredicate::always_false();
            assert_eq!(pred.name(), Some("always_false"));
            assert!(!pred.test(&5, &3));
            assert!(!pred.test(&-5, &-3));
            assert!(!pred.test(&0, &0));
            assert!(!pred.test(&100, &-100));
        }

        #[test]
        fn test_always_true_with_composition() {
            let always_true: RcBiPredicate<i32, i32> = RcBiPredicate::always_true();
            let positive_sum = |x: &i32, y: &i32| x + y > 0;

            // always_true AND something = something
            let combined = always_true.and(positive_sum);
            assert!(combined.test(&5, &3));
            assert!(!combined.test(&-5, &-3));
        }

        #[test]
        fn test_always_false_with_composition() {
            let always_false: RcBiPredicate<i32, i32> = RcBiPredicate::always_false();
            let positive_sum = |x: &i32, y: &i32| x + y > 0;

            // always_false OR something = something
            let combined = always_false.or(positive_sum);
            assert!(combined.test(&5, &3));
            assert!(!combined.test(&-5, &-3));
        }

        #[test]
        fn test_always_true_clone() {
            let pred: RcBiPredicate<i32, i32> = RcBiPredicate::always_true();
            let cloned = pred.clone();

            assert_eq!(cloned.name(), Some("always_true"));
            assert!(cloned.test(&5, &3));
            assert!(pred.test(&-5, &-3));
        }

        #[test]
        fn test_name_none() {
            let pred = RcBiPredicate::new(|x: &i32, y: &i32| x + y > 0);
            assert_eq!(pred.name(), None);
        }

        #[test]
        fn test_test_method() {
            let pred = RcBiPredicate::new(|x: &i32, y: &i32| *x > *y);
            assert!(pred.test(&10, &5));
            assert!(!pred.test(&3, &8));
        }

        #[test]
        fn test_clone() {
            let pred = RcBiPredicate::new(|x: &i32, y: &i32| x + y > 0);
            let cloned = pred.clone();

            assert!(pred.test(&5, &3));
            assert!(cloned.test(&5, &3));
        }

        #[test]
        fn test_clone_preserves_name() {
            let pred = RcBiPredicate::new_with_name("original", |x: &i32, y: &i32| x + y > 0);
            let cloned = pred.clone();

            assert_eq!(pred.name(), Some("original"));
            assert_eq!(cloned.name(), Some("original"));
        }

        #[test]
        fn test_and() {
            let sum_positive = RcBiPredicate::new(|x: &i32, y: &i32| x + y > 0);
            let first_positive = RcBiPredicate::new(|x: &i32, _y: &i32| *x > 0);

            let combined = sum_positive.clone().and(first_positive.clone());
            assert!(combined.test(&5, &3));
            assert!(!combined.test(&-5, &10));

            // Original predicates still usable
            assert!(sum_positive.test(&-5, &10));
        }

        #[test]
        fn test_or() {
            let sum_positive = RcBiPredicate::new(|x: &i32, y: &i32| x + y > 0);
            let first_positive = RcBiPredicate::new(|x: &i32, _y: &i32| *x > 0);

            let combined = sum_positive.or(first_positive.clone());
            assert!(combined.test(&5, &3));
            assert!(combined.test(&-5, &10));
        }

        #[test]
        fn test_not() {
            let sum_positive = RcBiPredicate::new(|x: &i32, y: &i32| x + y > 0);
            let sum_not_positive = !&sum_positive;

            assert!(!sum_not_positive.test(&5, &3));
            assert!(sum_not_positive.test(&-5, &-3));
        }

        #[test]
        fn test_xor() {
            let first_positive = RcBiPredicate::new(|x: &i32, _y: &i32| *x > 0);
            let second_positive = RcBiPredicate::new(|_x: &i32, y: &i32| *y > 0);

            let combined = first_positive.xor(second_positive);
            assert!(combined.test(&5, &-3));
            assert!(!combined.test(&5, &3));
        }

        #[test]
        fn test_nand() {
            let first_positive = RcBiPredicate::new(|x: &i32, _y: &i32| *x > 0);
            let second_positive = RcBiPredicate::new(|_x: &i32, y: &i32| *y > 0);

            let combined = first_positive.nand(second_positive);
            assert!(!combined.test(&5, &3));
            assert!(combined.test(&5, &-3));
        }

        #[test]
        fn test_nor() {
            let first_positive = RcBiPredicate::new(|x: &i32, _y: &i32| *x > 0);
            let second_positive = RcBiPredicate::new(|_x: &i32, y: &i32| *y > 0);

            let combined = first_positive.nor(second_positive);
            assert!(!combined.test(&5, &3));
            assert!(combined.test(&-5, &-3));
        }

        #[test]
        fn test_chain_combination() {
            let x_positive = RcBiPredicate::new(|x: &i32, _y: &i32| *x > 0);
            let y_positive = RcBiPredicate::new(|_x: &i32, y: &i32| *y > 0);

            let both_positive = x_positive.and(y_positive);
            assert!(both_positive.test(&5, &3));
            assert!(!both_positive.test(&5, &-3));
        }

        #[test]
        fn test_display() {
            let pred = RcBiPredicate::new_with_name("sum_positive", |x: &i32, y: &i32| x + y > 0);
            assert_eq!(format!("{}", pred), "RcBiPredicate(sum_positive)");
        }

        #[test]
        fn test_debug() {
            let pred = RcBiPredicate::new_with_name("test_pred", |x: &i32, y: &i32| x + y > 0);
            let debug_str = format!("{:?}", pred);
            assert!(debug_str.contains("RcBiPredicate"));
        }

        #[test]
        fn test_with_different_types() {
            let str_len_greater = RcBiPredicate::new(|s: &String, len: &usize| s.len() > *len);
            assert!(str_len_greater.test(&String::from("hello"), &3));
            assert!(!str_len_greater.test(&String::from("hi"), &5));
        }

        #[test]
        fn test_set_name() {
            let mut pred = RcBiPredicate::new(|x: &i32, y: &i32| x + y > 0);
            assert_eq!(pred.name(), None);

            pred.set_name("sum_positive");
            assert_eq!(pred.name(), Some("sum_positive"));
            assert!(pred.test(&5, &3));

            pred.set_name("updated_name");
            assert_eq!(pred.name(), Some("updated_name"));
        }
    }

    // ========================================================================
    // Conversion Tests - Test into_box, into_rc, into_arc
    // ========================================================================

    mod conversion_tests {
        use super::ArcBiPredicate;
        use super::BiPredicate;
        use super::BoxBiPredicate;

        #[test]
        fn test_struct_storing_arc_bi_predicate() {
            struct Validator {
                predicate: ArcBiPredicate<i32, i32>,
            }

            let validator = Validator {
                predicate: ArcBiPredicate::new(|x: &i32, y: &i32| x + y > 0),
            };

            assert!(validator.predicate.test(&5, &3));
        }

        #[test]
        fn test_struct_storing_box_bi_predicate() {
            struct Validator {
                predicate: BoxBiPredicate<i32, i32>,
            }

            let validator = Validator {
                predicate: BoxBiPredicate::new(|x: &i32, y: &i32| x + y > 0),
            };

            assert!(validator.predicate.test(&5, &3));
        }
    }

    // ========================================================================
    // Generic Constraint Tests - Test use with generic functions
    // ========================================================================
}
