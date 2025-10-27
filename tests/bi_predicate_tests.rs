/*******************************************************************************
 *
 *    Copyright (c) 2025.
 *    3-Prism Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

#[cfg(test)]
mod tests {
    use prism3_function::bi_predicate::{
        ArcBiPredicate,
        BiPredicate,
        BoxBiPredicate,
        FnBiPredicateOps,
        RcBiPredicate,
    };
    use std::thread;

    // ========================================================================
    // BiPredicate Trait Tests - Test closure and function pointer implementations
    // ========================================================================

    mod bi_predicate_trait_tests {
        use super::*;

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
    // FnBiPredicateOps Tests - Test extension methods for closures
    // ========================================================================

    mod bi_predicate_ext_tests {
        use super::*;

        #[test]
        fn test_closure_and() {
            let sum_positive = |x: &i32, y: &i32| x + y > 0;
            let first_positive = |x: &i32, _y: &i32| *x > 0;

            let combined = sum_positive.and(first_positive);
            assert!(combined.test(&5, &3)); // Both conditions met
            assert!(!combined.test(&-5, &10)); // Sum positive but first not
            assert!(!combined.test(&5, &-10)); // First positive but sum not
        }

        #[test]
        fn test_closure_or() {
            let sum_positive = |x: &i32, y: &i32| x + y > 0;
            let first_positive = |x: &i32, _y: &i32| *x > 0;

            let combined = sum_positive.or(first_positive);
            assert!(combined.test(&5, &3)); // Both conditions met
            assert!(combined.test(&-5, &10)); // Sum positive
            assert!(combined.test(&5, &-10)); // First positive
            assert!(!combined.test(&-5, &-10)); // Neither condition met
        }

        #[test]
        fn test_closure_not() {
            let sum_positive = |x: &i32, y: &i32| x + y > 0;
            let sum_not_positive = sum_positive.not();

            assert!(!sum_not_positive.test(&5, &3));
            assert!(sum_not_positive.test(&-5, &-3));
        }

        #[test]
        fn test_closure_xor() {
            let first_positive = |x: &i32, _y: &i32| *x > 0;
            let second_positive = |_x: &i32, y: &i32| *y > 0;

            let combined = first_positive.xor(second_positive);
            assert!(combined.test(&5, &-3)); // Only first positive
            assert!(combined.test(&-5, &3)); // Only second positive
            assert!(!combined.test(&5, &3)); // Both positive
            assert!(!combined.test(&-5, &-3)); // Neither positive
        }

        #[test]
        fn test_closure_nand() {
            let first_positive = |x: &i32, _y: &i32| *x > 0;
            let second_positive = |_x: &i32, y: &i32| *y > 0;

            let combined = first_positive.nand(second_positive);
            assert!(!combined.test(&5, &3)); // Both positive (NAND false)
            assert!(combined.test(&5, &-3)); // Only first positive
            assert!(combined.test(&-5, &3)); // Only second positive
            assert!(combined.test(&-5, &-3)); // Neither positive
        }

        #[test]
        fn test_closure_nor() {
            let first_positive = |x: &i32, _y: &i32| *x > 0;
            let second_positive = |_x: &i32, y: &i32| *y > 0;

            let combined = first_positive.nor(second_positive);
            assert!(!combined.test(&5, &3)); // Both positive
            assert!(!combined.test(&5, &-3)); // First positive
            assert!(!combined.test(&-5, &3)); // Second positive
            assert!(combined.test(&-5, &-3)); // Neither positive (NOR true)
        }

        #[test]
        fn test_closure_chain_combination() {
            let x_positive = |x: &i32, _y: &i32| *x > 0;
            let y_positive = |_x: &i32, y: &i32| *y > 0;
            let sum_large = |x: &i32, y: &i32| x + y > 100;

            let complex = x_positive.and(y_positive).or(sum_large);
            assert!(complex.test(&5, &3)); // Both positive
            assert!(complex.test(&50, &60)); // Sum large
            assert!(!complex.test(&5, &-3)); // Only first positive, sum not large
        }
    }

    // ========================================================================
    // BoxBiPredicate Tests
    // ========================================================================

    mod box_bi_predicate_tests {
        use super::*;

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
            let sum_not_positive = sum_positive.not();

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

    mod arc_bi_predicate_tests {
        use super::*;

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
            let sum_not_positive = sum_positive.not();

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
        fn test_to_box() {
            let arc_pred = ArcBiPredicate::new(|x: &i32, y: &i32| x + y > 0);
            let box_pred = arc_pred.to_box();
            assert!(box_pred.test(&5, &3));
            assert!(!box_pred.test(&-5, &-3));
        }

        #[test]
        fn test_to_box_preserves_name() {
            let arc_pred = ArcBiPredicate::new_with_name("test", |x: &i32, y: &i32| x + y > 0);
            let box_pred = arc_pred.to_box();
            assert_eq!(box_pred.name(), Some("test"));
            assert!(box_pred.test(&5, &3));
        }

        #[test]
        fn test_thread_safety() {
            let pred = ArcBiPredicate::new(|x: &i32, y: &i32| x + y > 0);

            let pred_clone = pred.clone();
            let handle = thread::spawn(move || pred_clone.test(&5, &3));

            assert!(pred.test(&10, &-5));
            assert!(handle.join().unwrap());
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
                let _ = handle.join().unwrap();
            }
        }
    }

    // ========================================================================
    // RcBiPredicate Tests
    // ========================================================================

    mod rc_bi_predicate_tests {
        use super::*;

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
            let sum_not_positive = sum_positive.not();

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

        #[test]
        fn test_to_box() {
            let rc_pred = RcBiPredicate::new(|x: &i32, y: &i32| x + y > 0);
            let box_pred = rc_pred.to_box();
            assert!(box_pred.test(&5, &3));
            assert!(!box_pred.test(&-5, &-3));
        }

        #[test]
        fn test_to_box_preserves_name() {
            let rc_pred = RcBiPredicate::new_with_name("test", |x: &i32, y: &i32| x + y > 0);
            let box_pred = rc_pred.to_box();
            assert_eq!(box_pred.name(), Some("test"));
            assert!(box_pred.test(&5, &3));
        }

        #[test]
        fn test_to_box_original_still_usable() {
            let rc_pred = RcBiPredicate::new(|x: &i32, y: &i32| x + y > 0);
            let _box_pred = rc_pred.to_box();

            // Original rc_pred should still be usable because to_box() doesn't consume it
            assert!(rc_pred.test(&5, &3));
            assert!(!rc_pred.test(&-5, &-3));
        }

        #[test]
        fn test_to_box_multiple_calls() {
            let rc_pred = RcBiPredicate::new(|x: &i32, y: &i32| x % 2 == 0 && y % 2 == 0);

            // Can call to_box() multiple times
            let box_pred1 = rc_pred.to_box();
            let box_pred2 = rc_pred.to_box();

            assert!(box_pred1.test(&2, &4));
            assert!(box_pred2.test(&4, &6));
            assert!(!box_pred1.test(&3, &4));
            assert!(!box_pred2.test(&2, &5));

            // Original still usable
            assert!(rc_pred.test(&2, &4));
        }

        #[test]
        fn test_to_box_with_different_types() {
            let rc_pred = RcBiPredicate::new(|s: &String, len: &usize| s.len() > *len);
            let box_pred = rc_pred.to_box();

            assert!(box_pred.test(&String::from("hello"), &3));
            assert!(!box_pred.test(&String::from("hi"), &5));

            // Original still usable
            assert!(rc_pred.test(&String::from("world"), &3));
        }

        #[test]
        fn test_to_box_and_composition() {
            let rc_pred = RcBiPredicate::new(|x: &i32, y: &i32| x + y > 0);
            let box_pred = rc_pred.to_box();

            // Compose with another predicate
            let both_positive = |x: &i32, y: &i32| *x > 0 && *y > 0;
            let combined = box_pred.and(both_positive);

            assert!(combined.test(&5, &3));
            assert!(!combined.test(&-5, &10));
        }
    }

    // ========================================================================
    // Conversion Tests - Test into_box, into_rc, into_arc
    // ========================================================================

    mod conversion_tests {
        use super::*;

        #[test]
        fn test_closure_into_box() {
            let closure = |x: &i32, y: &i32| x + y > 0;
            let box_pred: BoxBiPredicate<i32, i32> = closure.into_box();
            assert!(box_pred.test(&5, &3));
        }

        #[test]
        fn test_closure_into_rc() {
            let closure = |x: &i32, y: &i32| x + y > 0;
            let rc_pred: RcBiPredicate<i32, i32> = closure.into_rc();
            assert!(rc_pred.test(&5, &3));
        }

        #[test]
        fn test_closure_into_arc() {
            let closure = |x: &i32, y: &i32| x + y > 0;
            let arc_pred: ArcBiPredicate<i32, i32> = closure.into_arc();
            assert!(arc_pred.test(&5, &3));
        }

        #[test]
        fn test_box_to_box_zero_cost() {
            let pred = BoxBiPredicate::new(|x: &i32, y: &i32| x + y > 0);
            let same_pred = pred.into_box();
            assert!(same_pred.test(&5, &3));
        }

        #[test]
        fn test_box_to_rc() {
            let box_pred = BoxBiPredicate::new(|x: &i32, y: &i32| x + y > 0);
            let rc_pred = box_pred.into_rc();
            assert!(rc_pred.test(&5, &3));
        }

        #[test]
        fn test_box_to_rc_preserves_name() {
            let box_pred = BoxBiPredicate::new_with_name("test", |x: &i32, y: &i32| x + y > 0);
            let rc_pred = box_pred.into_rc();
            assert_eq!(rc_pred.name(), Some("test"));
        }

        #[test]
        fn test_arc_to_arc_zero_cost() {
            let pred = ArcBiPredicate::new(|x: &i32, y: &i32| x + y > 0);
            let same_pred = pred.into_arc();
            assert!(same_pred.test(&5, &3));
        }

        #[test]
        fn test_arc_to_box() {
            let arc_pred = ArcBiPredicate::new(|x: &i32, y: &i32| x + y > 0);
            let box_pred = arc_pred.into_box();
            assert!(box_pred.test(&5, &3));
        }

        #[test]
        fn test_arc_to_box_preserves_name() {
            let arc_pred = ArcBiPredicate::new_with_name("test", |x: &i32, y: &i32| x + y > 0);
            let box_pred = arc_pred.into_box();
            assert_eq!(box_pred.name(), Some("test"));
        }

        #[test]
        fn test_arc_to_rc() {
            let arc_pred = ArcBiPredicate::new(|x: &i32, y: &i32| x + y > 0);
            let rc_pred = arc_pred.into_rc();
            assert!(rc_pred.test(&5, &3));
        }

        #[test]
        fn test_arc_to_rc_preserves_name() {
            let arc_pred = ArcBiPredicate::new_with_name("test", |x: &i32, y: &i32| x + y > 0);
            let rc_pred = arc_pred.into_rc();
            assert_eq!(rc_pred.name(), Some("test"));
        }

        #[test]
        fn test_rc_to_rc_zero_cost() {
            let pred = RcBiPredicate::new(|x: &i32, y: &i32| x + y > 0);
            let same_pred = pred.into_rc();
            assert!(same_pred.test(&5, &3));
        }

        #[test]
        fn test_rc_to_box() {
            let rc_pred = RcBiPredicate::new(|x: &i32, y: &i32| x + y > 0);
            let box_pred = rc_pred.into_box();
            assert!(box_pred.test(&5, &3));
        }

        #[test]
        fn test_rc_to_box_preserves_name() {
            let rc_pred = RcBiPredicate::new_with_name("test", |x: &i32, y: &i32| x + y > 0);
            let box_pred = rc_pred.into_box();
            assert_eq!(box_pred.name(), Some("test"));
        }

        #[test]
        fn test_conversion_preserves_behavior() {
            let closure = |x: &i32, y: &i32| x + y > 10;

            let box_pred = closure.into_box();
            assert!(box_pred.test(&5, &6));
            assert!(!box_pred.test(&3, &4));

            let closure2 = |x: &i32, y: &i32| x + y > 10;
            let rc_pred = closure2.into_rc();
            assert!(rc_pred.test(&5, &6));
            assert!(!rc_pred.test(&3, &4));
        }

        #[test]
        fn test_conversion_chain() {
            let arc_pred = ArcBiPredicate::new_with_name("original", |x: &i32, y: &i32| x + y > 0);

            // Arc -> Rc
            let rc_pred = arc_pred.clone().into_rc();
            assert_eq!(rc_pred.name(), Some("original"));
            assert!(rc_pred.test(&5, &3));

            // Rc -> Box
            let box_pred = rc_pred.into_box();
            assert_eq!(box_pred.name(), Some("original"));
            assert!(box_pred.test(&5, &3));
        }

        #[test]
        fn test_arc_bi_predicate_new_instead_of_box() {
            // Demonstrate the correct way to create ArcBiPredicate
            let closure = |x: &i32, y: &i32| x + y > 0;
            let arc_pred = closure.into_arc();
            assert!(arc_pred.test(&5, &3));

            // Or directly
            let arc_pred2 = ArcBiPredicate::new(|x: &i32, y: &i32| x + y > 0);
            assert!(arc_pred2.test(&5, &3));
        }

        #[test]
        fn test_closure_to_arc_instead_of_box_to_arc() {
            // Right approach: convert closure directly to Arc
            let closure = |x: &i32, y: &i32| x + y > 0;
            let arc_pred = closure.into_arc();

            let arc_clone = arc_pred.clone();
            let handle = thread::spawn(move || arc_clone.test(&5, &3));

            assert!(handle.join().unwrap());
        }

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
    // into_fn Tests - Test conversion to FnMut for use with iterators
    // ========================================================================

    mod into_fn_tests {
        use super::*;

        #[test]
        fn test_closure_into_fn_with_filter() {
            let pairs = [(1, 2), (-1, 3), (5, -6), (3, 4)];
            let predicate = |x: &i32, y: &i32| x + y > 0;

            let result: Vec<_> = pairs
                .iter()
                .filter(|(x, y)| predicate.into_fn()(x, y))
                .copied()
                .collect();

            assert_eq!(result, vec![(1, 2), (-1, 3), (3, 4)]);
        }

        #[test]
        fn test_box_bi_predicate_into_fn_with_filter() {
            let pairs = [(1, 2), (-1, 3), (5, -6), (3, 4)];
            let predicate = BoxBiPredicate::new(|x: &i32, y: &i32| x + y > 0);

            let pred_fn = predicate.into_fn();
            let result: Vec<_> = pairs
                .iter()
                .filter(|(x, y)| pred_fn(x, y))
                .copied()
                .collect();

            assert_eq!(result, vec![(1, 2), (-1, 3), (3, 4)]);
        }

        #[test]
        fn test_arc_bi_predicate_into_fn_with_filter() {
            let pairs = [(1, 2), (-1, 3), (5, -6), (3, 4)];
            let predicate = ArcBiPredicate::new(|x: &i32, y: &i32| x + y > 0);

            let pred_fn = predicate.into_fn();
            let result: Vec<_> = pairs
                .iter()
                .filter(|(x, y)| pred_fn(x, y))
                .copied()
                .collect();

            assert_eq!(result, vec![(1, 2), (-1, 3), (3, 4)]);
        }

        #[test]
        fn test_rc_bi_predicate_into_fn_with_filter() {
            let pairs = [(1, 2), (-1, 3), (5, -6), (3, 4)];
            let predicate = RcBiPredicate::new(|x: &i32, y: &i32| x + y > 0);

            let pred_fn = predicate.into_fn();
            let result: Vec<_> = pairs
                .iter()
                .filter(|(x, y)| pred_fn(x, y))
                .copied()
                .collect();

            assert_eq!(result, vec![(1, 2), (-1, 3), (3, 4)]);
        }

        #[test]
        fn test_into_fn_with_complex_composition() {
            let x_positive = ArcBiPredicate::new(|x: &i32, _y: &i32| *x > 0);
            let y_positive = ArcBiPredicate::new(|_x: &i32, y: &i32| *y > 0);
            let predicate = x_positive.and(y_positive);

            let pairs = [(1, 2), (-1, 3), (5, -6), (3, 4)];
            let pred_fn = predicate.into_fn();
            let result: Vec<_> = pairs
                .iter()
                .filter(|(x, y)| pred_fn(x, y))
                .copied()
                .collect();

            assert_eq!(result, vec![(1, 2), (3, 4)]);
        }

        #[test]
        fn test_into_fn_preserves_closure_semantics() {
            let predicate = BoxBiPredicate::new(|x: &i32, y: &i32| *x > *y);
            let pred_fn = predicate.into_fn();

            assert!(pred_fn(&10, &5));
            assert!(!pred_fn(&3, &8));
            assert!(!pred_fn(&5, &5));
        }

        #[test]
        fn test_into_fn_with_partition() {
            let pairs = [(1, 2), (-1, 3), (5, -6), (3, 4)];
            let predicate = BoxBiPredicate::new(|x: &i32, y: &i32| x + y > 0);

            let pred_fn = predicate.into_fn();
            let positive: Vec<_> = pairs.iter().filter(|(x, y)| pred_fn(x, y)).collect();
            let negative: Vec<_> = pairs.iter().filter(|(x, y)| !pred_fn(x, y)).collect();

            assert_eq!(positive, vec![&(1, 2), &(-1, 3), &(3, 4)]);
            assert_eq!(negative, vec![&(5, -6)]);
        }

        #[test]
        fn test_into_fn_with_string() {
            let pairs = Vec::from([
                (String::from("hello"), 3),
                (String::from("hi"), 5),
                (String::from("world"), 4),
            ]);

            let predicate = BoxBiPredicate::new(|s: &String, len: &usize| s.len() > *len);
            let pred_fn = predicate.into_fn();

            let result: Vec<_> = pairs
                .iter()
                .filter(|(s, len)| pred_fn(s, len))
                .map(|(s, _)| s.clone())
                .collect();

            assert_eq!(result, vec![String::from("hello"), String::from("world")]);
        }

        #[test]
        fn test_into_fn_with_references() {
            let data = [(1, 2), (3, 4), (5, 6)];
            let predicate = RcBiPredicate::new(|x: &i32, y: &i32| x + y > 5);

            let pred_fn = predicate.into_fn();
            let count = data.iter().filter(|(x, y)| pred_fn(x, y)).count();

            assert_eq!(count, 2); // (3,4) and (5,6)
        }
    }

    // ========================================================================
    // Generic Constraint Tests - Test use with generic functions
    // ========================================================================

    mod generic_constraint_tests {
        use super::*;

        fn filter_pairs<P>(pairs: Vec<(i32, i32)>, predicate: &P) -> Vec<(i32, i32)>
        where
            P: BiPredicate<i32, i32>,
        {
            pairs
                .into_iter()
                .filter(|(x, y)| predicate.test(x, y))
                .collect()
        }

        #[test]
        fn test_generic_function_accepts_closure() {
            let pairs = vec![(1, 2), (-1, 3), (5, -6)];
            let result = filter_pairs(pairs, &|x: &i32, y: &i32| x + y > 0);
            assert_eq!(result, vec![(1, 2), (-1, 3)]);
        }

        #[test]
        fn test_generic_function_accepts_box_bi_predicate() {
            let pairs = vec![(1, 2), (-1, 3), (5, -6)];
            let pred = BoxBiPredicate::new(|x: &i32, y: &i32| x + y > 0);
            let result = filter_pairs(pairs, &pred);
            assert_eq!(result, vec![(1, 2), (-1, 3)]);
        }

        #[test]
        fn test_generic_function_accepts_arc_bi_predicate() {
            let pairs = vec![(1, 2), (-1, 3), (5, -6)];
            let pred = ArcBiPredicate::new(|x: &i32, y: &i32| x + y > 0);
            let result = filter_pairs(pairs, &pred);
            assert_eq!(result, vec![(1, 2), (-1, 3)]);
        }

        #[test]
        fn test_generic_function_accepts_rc_bi_predicate() {
            let pairs = vec![(1, 2), (-1, 3), (5, -6)];
            let pred = RcBiPredicate::new(|x: &i32, y: &i32| x + y > 0);
            let result = filter_pairs(pairs, &pred);
            assert_eq!(result, vec![(1, 2), (-1, 3)]);
        }

        #[test]
        fn test_generic_function_accepts_function_pointer() {
            fn sum_positive(x: &i32, y: &i32) -> bool {
                x + y > 0
            }

            let pairs = vec![(1, 2), (-1, 3), (5, -6)];
            let result = filter_pairs(pairs, &sum_positive);
            assert_eq!(result, vec![(1, 2), (-1, 3)]);
        }

        #[test]
        fn test_generic_count_with_different_bi_predicate_types() {
            fn count_matching<P>(pairs: &[(i32, i32)], pred: &P) -> usize
            where
                P: BiPredicate<i32, i32>,
            {
                pairs.iter().filter(|(x, y)| pred.test(x, y)).count()
            }

            let pairs = [(1, 2), (-1, 3), (5, -6), (3, 4)];

            let box_pred = BoxBiPredicate::new(|x: &i32, y: &i32| x + y > 0);
            assert_eq!(count_matching(&pairs, &box_pred), 3);

            let arc_pred = ArcBiPredicate::new(|x: &i32, y: &i32| x + y > 0);
            assert_eq!(count_matching(&pairs, &arc_pred), 3);

            let rc_pred = RcBiPredicate::new(|x: &i32, y: &i32| x + y > 0);
            assert_eq!(count_matching(&pairs, &rc_pred), 3);
        }

        #[test]
        fn test_generic_with_combined_bi_predicates() {
            let x_positive = ArcBiPredicate::new(|x: &i32, _y: &i32| *x > 0);
            let y_positive = ArcBiPredicate::new(|_x: &i32, y: &i32| *y > 0);
            let combined = x_positive.and(y_positive);

            let pairs = [(1, 2), (-1, 3), (5, -6), (3, 4)];
            let result = filter_pairs(pairs.to_vec(), &combined);
            assert_eq!(result, vec![(1, 2), (3, 4)]);
        }

        #[test]
        fn test_generic_with_string_bi_predicates() {
            fn filter_string_pairs<P>(
                pairs: Vec<(String, usize)>,
                predicate: &P,
            ) -> Vec<(String, usize)>
            where
                P: BiPredicate<String, usize>,
            {
                pairs
                    .into_iter()
                    .filter(|(s, len)| predicate.test(s, len))
                    .collect()
            }

            let pairs = vec![
                (String::from("hello"), 3),
                (String::from("hi"), 5),
                (String::from("world"), 4),
            ];

            let pred = BoxBiPredicate::new(|s: &String, len: &usize| s.len() > *len);
            let result = filter_string_pairs(pairs, &pred);
            assert_eq!(result.len(), 2);
        }

        #[test]
        fn test_bi_predicate_as_struct_field() {
            struct Validator<P> {
                predicate: P,
            }

            impl<P> Validator<P>
            where
                P: BiPredicate<i32, i32>,
            {
                fn validate(&self, x: i32, y: i32) -> bool {
                    self.predicate.test(&x, &y)
                }
            }

            let validator = Validator {
                predicate: BoxBiPredicate::new(|x: &i32, y: &i32| x + y > 0),
            };

            assert!(validator.validate(5, 3));
            assert!(!validator.validate(-5, -3));
        }

        #[test]
        fn test_returning_bi_predicate_from_function() {
            fn create_sum_checker(threshold: i32) -> impl BiPredicate<i32, i32> {
                move |x: &i32, y: &i32| x + y > threshold
            }

            let checker = create_sum_checker(10);
            assert!(checker.test(&6, &5));
            assert!(!checker.test(&3, &4));
        }

        #[test]
        fn test_thread_safety_with_arc_bi_predicate() {
            fn process_in_thread<P>(pred: P, x: i32, y: i32) -> bool
            where
                P: BiPredicate<i32, i32> + Send + 'static,
            {
                thread::spawn(move || pred.test(&x, &y)).join().unwrap()
            }

            let pred = ArcBiPredicate::new(|x: &i32, y: &i32| x + y > 0);
            assert!(process_in_thread(pred, 5, 3));
        }

        #[test]
        fn test_mixed_bi_predicate_types_in_sequence() {
            let pairs = [(1, 2), (-1, 3), (5, -6), (3, 4)];

            // Use different types in sequence
            let box_pred = BoxBiPredicate::new(|x: &i32, y: &i32| x + y > 0);
            let count1 = pairs.iter().filter(|(x, y)| box_pred.test(x, y)).count();

            let arc_pred = ArcBiPredicate::new(|x: &i32, _y: &i32| *x > 0);
            let count2 = pairs.iter().filter(|(x, y)| arc_pred.test(x, y)).count();

            assert_eq!(count1, 3);
            assert_eq!(count2, 3);
        }

        #[test]
        fn test_generic_with_custom_types() {
            #[derive(Debug, Clone, PartialEq)]
            struct Point {
                x: i32,
                y: i32,
            }

            fn filter_points<P>(points: Vec<(Point, Point)>, pred: &P) -> Vec<(Point, Point)>
            where
                P: BiPredicate<Point, Point>,
            {
                points
                    .into_iter()
                    .filter(|(p1, p2)| pred.test(p1, p2))
                    .collect()
            }

            let points = vec![
                (Point { x: 1, y: 2 }, Point { x: 3, y: 4 }),
                (Point { x: -1, y: 2 }, Point { x: 1, y: -4 }),
            ];

            let pred = BoxBiPredicate::new(|p1: &Point, p2: &Point| p1.x + p2.x > 0);
            let result = filter_points(points, &pred);
            assert_eq!(result.len(), 1);
        }
    }

    // ========================================================================
    // Default Implementation Tests - Test that custom types can use
    // default implementations of into_xxx methods
    // ========================================================================

    mod default_implementation_tests {
        use super::*;

        // Custom bi-predicate type that only implements the core
        // test method and relies on default implementations for
        // all conversion methods
        #[derive(Clone)]
        struct CustomBiPredicate<T, U>
        where
            T: 'static,
            U: 'static,
        {
            threshold: i32,
            _phantom: std::marker::PhantomData<(T, U)>,
        }

        impl CustomBiPredicate<i32, i32> {
            fn new(threshold: i32) -> Self {
                Self {
                    threshold,
                    _phantom: std::marker::PhantomData,
                }
            }
        }

        // Only implement the core test method - all into_xxx and to_xxx
        // methods will use default implementations
        impl BiPredicate<i32, i32> for CustomBiPredicate<i32, i32> {
            fn test(&self, first: &i32, second: &i32) -> bool {
                first + second > self.threshold
            }

            // All other methods (into_box, into_rc, into_arc, into_fn,
            // to_box, to_rc, to_arc, to_fn) use default implementations automatically
        }

        #[test]
        fn test_custom_type_basic_test() {
            let pred = CustomBiPredicate::new(10);
            assert!(pred.test(&6, &5));
            assert!(pred.test(&10, &1));
            assert!(!pred.test(&5, &5));
            assert!(!pred.test(&3, &4));
        }

        #[test]
        fn test_custom_type_into_box() {
            let pred = CustomBiPredicate::new(10);
            // This uses the default implementation
            let box_pred = pred.into_box();

            assert!(box_pred.test(&6, &5));
            assert!(box_pred.test(&10, &1));
            assert!(!box_pred.test(&5, &5));
            assert!(!box_pred.test(&3, &4));
        }

        #[test]
        fn test_custom_type_into_rc() {
            let pred = CustomBiPredicate::new(10);
            // This uses the default implementation
            let rc_pred = pred.into_rc();

            assert!(rc_pred.test(&6, &5));
            assert!(rc_pred.test(&10, &1));
            assert!(!rc_pred.test(&5, &5));
            assert!(!rc_pred.test(&3, &4));

            // Verify it can be cloned (RcBiPredicate feature)
            let cloned = rc_pred.clone();
            assert!(cloned.test(&6, &5));
            assert!(rc_pred.test(&6, &5));
        }

        #[test]
        fn test_custom_type_into_arc() {
            // Custom type for thread-safe testing
            struct ThreadSafePredicate {
                threshold: i32,
            }

            impl ThreadSafePredicate {
                fn new(threshold: i32) -> Self {
                    Self { threshold }
                }
            }

            // Implement Send + Sync to allow conversion to Arc
            unsafe impl Send for ThreadSafePredicate {}
            unsafe impl Sync for ThreadSafePredicate {}

            // Only implement test method
            impl BiPredicate<i32, i32> for ThreadSafePredicate {
                fn test(&self, first: &i32, second: &i32) -> bool {
                    first + second > self.threshold
                }
            }

            let pred = ThreadSafePredicate::new(10);
            // This uses the default implementation
            let arc_pred = pred.into_arc();

            assert!(arc_pred.test(&6, &5));
            assert!(arc_pred.test(&10, &1));
            assert!(!arc_pred.test(&5, &5));
            assert!(!arc_pred.test(&3, &4));

            // Verify it can be sent across threads
            let arc_clone = arc_pred.clone();
            let handle = thread::spawn(move || arc_clone.test(&6, &5));

            assert!(handle.join().unwrap());
            assert!(arc_pred.test(&10, &1));
        }

        #[test]
        fn test_custom_type_into_fn() {
            let pred = CustomBiPredicate::new(10);
            // This uses the default implementation
            let func = pred.into_fn();

            assert!(func(&6, &5));
            assert!(func(&10, &1));
            assert!(!func(&5, &5));
            assert!(!func(&3, &4));
        }

        #[test]
        fn test_custom_type_into_fn_with_filter() {
            let pred = CustomBiPredicate::new(10);
            let func = pred.into_fn();

            let pairs = [(6, 5), (3, 4), (10, 1), (5, 5)];
            let result: Vec<_> = pairs.iter().filter(|(x, y)| func(x, y)).collect();

            assert_eq!(result, vec![&(6, 5), &(10, 1)]);
        }

        #[test]
        fn test_custom_type_can_be_used_in_generic_context() {
            fn accepts_predicate<P>(pred: &P, x: i32, y: i32) -> bool
            where
                P: BiPredicate<i32, i32>,
            {
                pred.test(&x, &y)
            }

            let pred = CustomBiPredicate::new(10);
            assert!(accepts_predicate(&pred, 6, 5));
            assert!(!accepts_predicate(&pred, 3, 4));
        }

        #[test]
        fn test_custom_type_composition_via_conversion() {
            let custom_pred = CustomBiPredicate::new(10);
            let box_pred = custom_pred.into_box();

            // Compose with another predicate
            let both_positive = |x: &i32, y: &i32| *x > 0 && *y > 0;
            let combined = box_pred.and(both_positive);

            assert!(combined.test(&6, &5)); // Sum > 10, both positive
            assert!(!combined.test(&-6, &20)); // Sum > 10, but not both
            assert!(!combined.test(&3, &4)); // Both positive, but sum <= 10
        }

        #[test]
        fn test_custom_type_all_conversions_preserve_behavior() {
            let threshold = 10;

            let custom_pred1 = CustomBiPredicate::new(threshold);
            let box_pred = custom_pred1.into_box();

            let custom_pred2 = CustomBiPredicate::new(threshold);
            let rc_pred = custom_pred2.into_rc();

            let test_values = [(6, 5), (3, 4), (10, 1), (5, 5)];

            // All converted predicates should behave the same
            for (x, y) in &test_values {
                let expected = x + y > threshold;
                assert_eq!(box_pred.test(x, y), expected);
                assert_eq!(rc_pred.test(x, y), expected);
            }
        }

        // ========================================================================
        // Test default to_xxx implementations
        // ========================================================================

        #[test]
        fn test_custom_type_to_box() {
            let pred = CustomBiPredicate::new(10);
            // This uses the default implementation
            let box_pred = pred.to_box();

            assert!(box_pred.test(&6, &5));
            assert!(box_pred.test(&10, &1));
            assert!(!box_pred.test(&5, &5));
            assert!(!box_pred.test(&3, &4));
        }

        #[test]
        fn test_custom_type_to_rc() {
            let pred = CustomBiPredicate::new(10);
            // This uses the default implementation
            let rc_pred = pred.to_rc();

            assert!(rc_pred.test(&6, &5));
            assert!(rc_pred.test(&10, &1));
            assert!(!rc_pred.test(&5, &5));
            assert!(!rc_pred.test(&3, &4));

            // Verify it can be cloned (RcBiPredicate feature)
            let cloned = rc_pred.clone();
            assert!(cloned.test(&6, &5));
            assert!(rc_pred.test(&6, &5));
        }

        #[test]
        fn test_custom_type_to_arc() {
            // Custom type for thread-safe testing
            #[derive(Clone)]
            struct ThreadSafePredicate {
                threshold: i32,
            }

            impl ThreadSafePredicate {
                fn new(threshold: i32) -> Self {
                    Self { threshold }
                }
            }

            // Implement Send + Sync to allow conversion to Arc
            unsafe impl Send for ThreadSafePredicate {}
            unsafe impl Sync for ThreadSafePredicate {}

            // Only implement test method
            impl BiPredicate<i32, i32> for ThreadSafePredicate {
                fn test(&self, first: &i32, second: &i32) -> bool {
                    first + second > self.threshold
                }
            }

            let pred = ThreadSafePredicate::new(10);
            // This uses the default implementation
            let arc_pred = pred.to_arc();

            assert!(arc_pred.test(&6, &5));
            assert!(arc_pred.test(&10, &1));
            assert!(!arc_pred.test(&5, &5));
            assert!(!arc_pred.test(&3, &4));

            // Verify it can be sent across threads
            let arc_clone = arc_pred.clone();
            let handle = thread::spawn(move || arc_clone.test(&6, &5));

            assert!(handle.join().unwrap());
            assert!(arc_pred.test(&10, &1));
        }

        #[test]
        fn test_custom_type_to_fn() {
            let pred = CustomBiPredicate::new(10);
            // This uses the default implementation
            let func = pred.to_fn();

            assert!(func(&6, &5));
            assert!(func(&10, &1));
            assert!(!func(&5, &5));
            assert!(!func(&3, &4));
        }

        #[test]
        fn test_custom_type_to_fn_with_filter() {
            let pred = CustomBiPredicate::new(10);
            let func = pred.to_fn();

            let pairs = [(6, 5), (3, 4), (10, 1), (5, 5)];
            let result: Vec<_> = pairs.iter().filter(|(x, y)| func(x, y)).collect();

            assert_eq!(result, vec![&(6, 5), &(10, 1)]);
        }

        #[test]
        fn test_custom_type_original_still_usable_after_to_box() {
            let pred = CustomBiPredicate::new(10);
            let _box_pred = pred.to_box();

            // Original pred should still be usable because to_box() clones it
            assert!(pred.test(&6, &5));
            assert!(!pred.test(&3, &4));
        }

        #[test]
        fn test_custom_type_original_still_usable_after_to_rc() {
            let pred = CustomBiPredicate::new(10);
            let _rc_pred = pred.to_rc();

            // Original pred should still be usable because to_rc() clones it
            assert!(pred.test(&6, &5));
            assert!(!pred.test(&3, &4));
        }

        #[test]
        fn test_custom_type_original_still_usable_after_to_fn() {
            let pred = CustomBiPredicate::new(10);
            let _func = pred.to_fn();

            // Original pred should still be usable because to_fn() clones it
            assert!(pred.test(&6, &5));
            assert!(!pred.test(&3, &4));
        }

        #[test]
        fn test_custom_type_all_to_conversions_preserve_behavior() {
            let threshold = 10;
            let pred = CustomBiPredicate::new(threshold);

            let box_pred = pred.to_box();
            let rc_pred = pred.to_rc();

            let test_values = [(6, 5), (3, 4), (10, 1), (5, 5)];

            // All converted predicates should behave the same
            for (x, y) in &test_values {
                let expected = x + y > threshold;
                assert_eq!(pred.test(x, y), expected);
                assert_eq!(box_pred.test(x, y), expected);
                assert_eq!(rc_pred.test(x, y), expected);
            }
        }
    }

    // ========================================================================
    // Edge Case Tests
    // ========================================================================

    mod edge_case_tests {
        use super::*;

        #[test]
        fn test_with_zero() {
            let sum_positive = BoxBiPredicate::new(|x: &i32, y: &i32| x + y > 0);
            assert!(!sum_positive.test(&0, &0));
            assert!(sum_positive.test(&1, &0));
            assert!(sum_positive.test(&0, &1));
        }

        #[test]
        fn test_always_true() {
            let always_true = BoxBiPredicate::new(|_x: &i32, _y: &i32| true);
            assert!(always_true.test(&5, &3));
            assert!(always_true.test(&-5, &-3));
            assert!(always_true.test(&0, &0));
        }

        #[test]
        fn test_always_false() {
            let always_false = BoxBiPredicate::new(|_x: &i32, _y: &i32| false);
            assert!(!always_false.test(&5, &3));
            assert!(!always_false.test(&-5, &-3));
            assert!(!always_false.test(&0, &0));
        }

        #[test]
        fn test_double_negation() {
            let sum_positive = BoxBiPredicate::new(|x: &i32, y: &i32| x + y > 0);
            let not_not = sum_positive.not().not();
            assert!(not_not.test(&5, &3));
            assert!(!not_not.test(&-5, &-3));
        }

        #[test]
        fn test_with_empty_string() {
            let is_empty =
                BoxBiPredicate::new(|s1: &String, s2: &String| s1.is_empty() && s2.is_empty());
            assert!(is_empty.test(&String::new(), &String::new()));
            assert!(!is_empty.test(&String::from("a"), &String::new()));
        }

        #[test]
        fn test_with_large_numbers() {
            let sum_overflow_safe =
                BoxBiPredicate::new(|x: &i64, y: &i64| x.checked_add(*y).is_some());
            let max_minus_one = i64::MAX - 1;
            assert!(sum_overflow_safe.test(&max_minus_one, &1));
            assert!(!sum_overflow_safe.test(&i64::MAX, &1));
        }

        #[test]
        fn test_with_floating_point() {
            let close_enough = BoxBiPredicate::new(|x: &f64, y: &f64| (*x - *y).abs() < 0.01);
            assert!(close_enough.test(&1.0, &1.005));
            assert!(!close_enough.test(&1.0, &1.02));
        }

        #[test]
        fn test_complex_chain() {
            let p1 = BoxBiPredicate::new(|x: &i32, _y: &i32| *x > 0);
            let p2 = BoxBiPredicate::new(|_x: &i32, y: &i32| *y > 0);
            let p3 = BoxBiPredicate::new(|x: &i32, y: &i32| x + y > 10);

            let complex = p1.and(p2).or(p3);
            assert!(complex.test(&5, &3)); // Both positive
            assert!(complex.test(&50, &-30)); // Sum > 10 (50 + (-30) = 20)
            assert!(!complex.test(&-5, &3)); // Not both positive, sum not > 10
        }
    }

    // ========================================================================
    // Mixed Type Combination Tests
    // ========================================================================

    mod mixed_type_combination_tests {
        use super::*;

        #[test]
        fn test_closure_to_box() {
            let closure = |x: &i32, y: &i32| x + y > 0;
            let box_pred = closure.and(|x: &i32, _y: &i32| *x > 0);
            assert!(box_pred.test(&5, &3));
            assert!(!box_pred.test(&-5, &10));
        }

        #[test]
        fn test_box_with_closure() {
            let box_pred = BoxBiPredicate::new(|x: &i32, y: &i32| x + y > 0);
            let combined = box_pred.and(|x: &i32, _y: &i32| *x > 0);
            assert!(combined.test(&5, &3));
            assert!(!combined.test(&-5, &10));
        }

        #[test]
        fn test_arc_preserves_original() {
            let arc1 = ArcBiPredicate::new(|x: &i32, y: &i32| x + y > 0);
            let arc2 = ArcBiPredicate::new(|x: &i32, _y: &i32| *x > 0);

            let _combined = arc1.clone().and(arc2.clone());

            // Originals still usable
            assert!(arc1.test(&-5, &10));
            assert!(arc2.test(&5, &-10));
        }

        #[test]
        fn test_rc_preserves_original() {
            let rc1 = RcBiPredicate::new(|x: &i32, y: &i32| x + y > 0);
            let rc2 = RcBiPredicate::new(|x: &i32, _y: &i32| *x > 0);

            let _combined = rc1.clone().and(rc2.clone());

            // Originals still usable
            assert!(rc1.test(&-5, &10));
            assert!(rc2.test(&5, &-10));
        }
    }
}

#[cfg(test)]
mod to_fn_tests {
    use prism3_function::bi_predicate::{
        ArcBiPredicate,
        BiPredicate,
        RcBiPredicate,
    };

    #[test]
    fn test_rc_to_fn() {
        let pred = RcBiPredicate::new(|x: &i32, y: &i32| x + y > 0);
        let func = pred.to_fn();

        assert!(func(&5, &3));
        assert!(!func(&-5, &-3));
        assert!(!func(&0, &0));
    }

    #[test]
    fn test_rc_to_fn_multiple_calls() {
        let pred = RcBiPredicate::new(|x: &i32, y: &i32| x % 2 == 0 && y % 2 == 0);
        let func = pred.to_fn();

        assert!(func(&2, &4));
        assert!(func(&4, &6));
        assert!(!func(&3, &4));
        assert!(!func(&2, &5));
    }

    #[test]
    fn test_arc_to_fn() {
        let pred = ArcBiPredicate::new(|x: &i32, y: &i32| x + y > 0);
        let func = pred.to_fn();

        assert!(func(&5, &3));
        assert!(!func(&-5, &-3));
        assert!(!func(&0, &0));
    }

    #[test]
    fn test_arc_to_fn_multiple_calls() {
        let pred = ArcBiPredicate::new(|x: &i32, y: &i32| x % 2 == 0 && y % 2 == 0);
        let func = pred.to_fn();

        assert!(func(&2, &4));
        assert!(func(&4, &6));
        assert!(!func(&3, &4));
        assert!(!func(&2, &5));
    }

    #[test]
    fn test_rc_to_fn_with_composition() {
        let is_sum_positive = RcBiPredicate::new(|x: &i32, y: &i32| x + y > 0);
        let is_both_even = RcBiPredicate::new(|x: &i32, y: &i32| x % 2 == 0 && y % 2 == 0);

        let combined = is_sum_positive.and(is_both_even);
        let func = combined.to_fn();

        assert!(func(&2, &4));
        assert!(!func(&1, &3));
        assert!(!func(&-2, &-4));
    }

    #[test]
    fn test_arc_to_fn_with_composition() {
        let is_sum_positive = ArcBiPredicate::new(|x: &i32, y: &i32| x + y > 0);
        let is_both_even = ArcBiPredicate::new(|x: &i32, y: &i32| x % 2 == 0 && y % 2 == 0);

        let combined = is_sum_positive.and(is_both_even);
        let func = combined.to_fn();

        assert!(func(&2, &4));
        assert!(!func(&1, &3));
        assert!(!func(&-2, &-4));
    }

    #[test]
    fn test_rc_to_rc() {
        let pred = RcBiPredicate::new(|x: &i32, y: &i32| x + y > 0);
        let rc_pred = pred.to_rc();

        assert!(rc_pred.test(&5, &3));
        assert!(!rc_pred.test(&-5, &-3));
        assert!(pred.test(&5, &3));
    }

    #[test]
    fn test_arc_to_arc() {
        let pred = ArcBiPredicate::new(|x: &i32, y: &i32| x + y > 0);
        let arc_pred = pred.to_arc();

        assert!(arc_pred.test(&5, &3));
        assert!(!arc_pred.test(&-5, &-3));
        assert!(pred.test(&5, &3));
    }

    #[test]
    fn test_rc_to_rc_preserves_name() {
        let pred = RcBiPredicate::new_with_name("test_pred", |x: &i32, y: &i32| x + y > 0);
        let rc_pred = pred.to_rc();

        assert_eq!(rc_pred.name(), Some("test_pred"));
        assert!(rc_pred.test(&5, &3));
    }

    #[test]
    fn test_arc_to_arc_preserves_name() {
        let pred = ArcBiPredicate::new_with_name("test_pred", |x: &i32, y: &i32| x + y > 0);
        let arc_pred = pred.to_arc();

        assert_eq!(arc_pred.name(), Some("test_pred"));
        assert!(arc_pred.test(&5, &3));
    }

    #[test]
    fn test_closure_to_box() {
        let closure = |x: &i32, y: &i32| x + y > 0;
        let box_pred = closure.to_box();

        assert!(box_pred.test(&5, &3));
        assert!(!box_pred.test(&-5, &-3));
    }

    #[test]
    fn test_closure_to_rc() {
        let closure = |x: &i32, y: &i32| x + y > 0;
        let rc_pred = closure.to_rc();

        assert!(rc_pred.test(&5, &3));
        assert!(!rc_pred.test(&-5, &-3));
    }

    #[test]
    fn test_closure_to_arc() {
        let closure = |x: &i32, y: &i32| x + y > 0;
        let arc_pred = closure.to_arc();

        assert!(arc_pred.test(&5, &3));
        assert!(!arc_pred.test(&-5, &-3));
    }

    #[test]
    fn test_closure_to_fn() {
        let closure = |x: &i32, y: &i32| x + y > 0;
        let func = closure.to_fn();

        assert!(func(&5, &3));
        assert!(!func(&-5, &-3));
    }

    #[test]
    fn test_closure_to_box_multiple_calls() {
        let closure = |x: &i32, y: &i32| x % 2 == 0 && y % 2 == 0;

        let box_pred1 = closure.to_box();
        let box_pred2 = closure.to_box();

        assert!(box_pred1.test(&2, &4));
        assert!(box_pred2.test(&4, &6));
        assert!(!box_pred1.test(&3, &4));
    }

    #[test]
    fn test_closure_to_rc_can_clone() {
        let closure = |x: &i32, y: &i32| x + y > 0;
        let rc_pred = closure.to_rc();
        let cloned = rc_pred.clone();

        assert!(rc_pred.test(&5, &3));
        assert!(cloned.test(&5, &3));
        assert!(!rc_pred.test(&-5, &-3));
        assert!(!cloned.test(&-5, &-3));
    }

    #[test]
    fn test_closure_to_arc_thread_safe() {
        let closure = |x: &i32, y: &i32| x + y > 0;
        let arc_pred = closure.to_arc();
        let arc_clone = arc_pred.clone();

        let handle = std::thread::spawn(move || arc_clone.test(&5, &3));

        assert!(arc_pred.test(&10, &5));
        assert!(handle.join().unwrap());
    }

    #[test]
    fn test_closure_to_fn_with_filter() {
        let closure = |x: &i32, y: &i32| x + y > 0;
        let func = closure.to_fn();

        let pairs = [(1, 2), (-1, 3), (5, -6), (3, 4)];
        let result: Vec<_> = pairs.iter().filter(|(x, y)| func(x, y)).collect();

        assert_eq!(result, vec![&(1, 2), &(-1, 3), &(3, 4)]);
    }
}
